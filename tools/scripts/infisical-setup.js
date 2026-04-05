/**
 * Infisical programmatic setup script.
 * Runs inside the nyx-infisical container.
 * Usage: docker exec nyx-infisical node /workspace/tools/scripts/infisical-setup.js
 */
const http = require('http');
const jsrp = require('/backend/node_modules/jsrp');
const argon2 = require('/backend/node_modules/argon2');
const nacl = require('/backend/node_modules/tweetnacl');
const naclUtil = require('/backend/node_modules/tweetnacl-util');
const nodeCrypto = require('crypto');

// ── Admin account ─────────────────────────────────────────────────────────────
// Read from .secrets/bootstrap.env — never hardcoded.
function requireEnv(name) {
  const val = process.env[name];
  if (!val) throw new Error(
    `Required env var "${name}" is not set.\n` +
    `Source .secrets/bootstrap.env and .secrets/secrets.env before running.`
  );
  return val;
}

const EMAIL      = requireEnv('INFISICAL_ADMIN_EMAIL');
const PASSWORD   = requireEnv('INFISICAL_ADMIN_PASSWORD');
const FIRST_NAME = process.env.INFISICAL_ADMIN_FIRST_NAME || 'Nyx';
const LAST_NAME  = process.env.INFISICAL_ADMIN_LAST_NAME  || 'Admin';
const ORG_NAME   = process.env.INFISICAL_ORG_NAME         || 'Nyx';

// ── Secrets to push to Infisical ──────────────────────────────────────────────
// All values sourced from environment at runtime. No defaults for sensitive vars.
function getSecrets() {
  return {
    // DB passwords — must match .secrets/bootstrap.env
    POSTGRES_PASSWORD:         requireEnv('POSTGRES_ROOT_PASSWORD'),  // bootstrap var name
    NYX_APP_DB_PASSWORD:       requireEnv('NYX_APP_DB_PASSWORD'),
    NYX_MIGRATION_DB_PASSWORD: requireEnv('NYX_MIGRATION_DB_PASSWORD'),
    KRATOS_DB_PASSWORD:        requireEnv('KRATOS_DB_PASSWORD'),
    INFISICAL_DB_PASSWORD:     requireEnv('INFISICAL_DB_PASSWORD'),
    // Kratos secrets
    KRATOS_COOKIE_SECRET:      requireEnv('KRATOS_COOKIE_SECRET'),
    KRATOS_CIPHER_SECRET:      requireEnv('KRATOS_CIPHER_SECRET'),
    // JWT
    JWT_SECRET:                requireEnv('JWT_SECRET'),
    // SMTP — must be a real provider, never Mailhog
    SMTP_CONNECTION_URI:       requireEnv('SMTP_CONNECTION_URI'),
    SMTP_FROM_ADDRESS:         requireEnv('SMTP_FROM_ADDRESS'),
    // CORS (comma-separated, for Heimdall)
    CORS_ALLOWED_ORIGINS:      requireEnv('CORS_ALLOWED_ORIGINS'),
    // App URLs — injected into Kratos config via env var substitution in kratos.yml
    NX_WEB_URL:                requireEnv('NX_WEB_URL'),
    EDGE_URL:                  requireEnv('EDGE_URL'),
    COOKIE_DOMAIN:             requireEnv('COOKIE_DOMAIN'),
    KRATOS_PUBLIC_URL:         requireEnv('KRATOS_PUBLIC_URL'),
    // Google OAuth (optional — empty string disables)
    GOOGLE_CLIENT_ID:          process.env.GOOGLE_CLIENT_ID    || '',
    GOOGLE_CLIENT_SECRET:      process.env.GOOGLE_CLIENT_SECRET || '',
    // Infrastructure
    POSTGRES_HOST:             process.env.POSTGRES_HOST || 'postgres',
  };
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

function req(method, path, body, token) {
  return new Promise((resolve, reject) => {
    const data = body ? JSON.stringify(body) : null;
    const headers = {
      'Content-Type': 'application/json',
      'User-Agent':   'nyx-setup/1.0',
    };
    if (data) headers['Content-Length'] = Buffer.byteLength(data);
    if (token) headers['Authorization'] = `Bearer ${token}`;
    const r = http.request({ hostname: 'localhost', port: 8080, path, method, headers }, res => {
      let out = '';
      res.on('data', d => out += d);
      res.on('end', () => {
        try { resolve({ status: res.statusCode, body: JSON.parse(out) }); }
        catch { resolve({ status: res.statusCode, body: out }); }
      });
    });
    r.on('error', reject);
    if (data) r.write(data);
    r.end();
  });
}

// ── AES-256-GCM ────────────────────────────────────────────────────────────────

function aesEncrypt(plaintext, keyBase64) {
  const key = Buffer.from(keyBase64, 'base64');
  const iv  = nodeCrypto.randomBytes(12);
  const cipher = nodeCrypto.createCipheriv('aes-256-gcm', key, iv);
  const enc = Buffer.concat([cipher.update(plaintext, 'utf8'), cipher.final()]);
  return {
    ciphertext: enc.toString('base64'),
    iv:         iv.toString('base64'),
    tag:        cipher.getAuthTag().toString('base64'),
  };
}

// ── SRP key generation (mirrors generateUserSrpKeys in the Infisical codebase) ──

async function generateSrpKeys(email, password) {
  const keyPair    = nacl.box.keyPair();
  const publicKey  = naclUtil.encodeBase64(keyPair.publicKey);
  const privateKey = naclUtil.encodeBase64(keyPair.secretKey);

  const client = new jsrp.client();
  await new Promise(res => client.init({ username: email, password }, res));
  const { salt, verifier } = await new Promise((res, rej) =>
    client.createVerifier((err, r) => err ? rej(err) : res(r))
  );

  const derivedKey = await argon2.hash(password, {
    salt: Buffer.from(salt), memoryCost: 65536, timeCost: 3,
    parallelism: 1, hashLength: 32, type: argon2.argon2id, raw: true,
  });
  const derivedKeyB64 = derivedKey.toString('base64');

  const symmetricKey = nodeCrypto.randomBytes(32);
  const encPriv = aesEncrypt(privateKey, symmetricKey.toString('base64'));
  const encSym  = aesEncrypt(symmetricKey.toString('hex'), derivedKeyB64);

  return {
    publicKey, salt, verifier,
    encryptedPrivateKey:    encPriv.ciphertext,
    encryptedPrivateKeyIV:  encPriv.iv,
    encryptedPrivateKeyTag: encPriv.tag,
    protectedKey:           encSym.ciphertext,
    protectedKeyIV:         encSym.iv,
    protectedKeyTag:        encSym.tag,
  };
}

// ── SRP login ─────────────────────────────────────────────────────────────────

async function srpLogin(email, password) {
  const client = new jsrp.client();
  await new Promise(res => client.init({ username: email, password }, res));

  const r1 = await req('POST', '/api/v1/auth/login1', { email, clientPublicKey: client.getPublicKey() });
  if (r1.status !== 200) throw new Error('login1 failed: ' + JSON.stringify(r1.body));

  client.setSalt(r1.body.salt);
  client.setServerPublicKey(r1.body.serverPublicKey);

  const r2 = await req('POST', '/api/v1/auth/login2', { email, clientProof: client.getProof() });
  if (r2.status !== 200) throw new Error('login2 failed: ' + JSON.stringify(r2.body));
  return r2.body.token;
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  const SECRETS = getSecrets();   // resolves from process.env — throws on missing vars

  // ── 1. Admin signup (only works on a fresh instance) ──
  let token;
  let orgId;

  // Try login first; if that fails (account doesn't exist), do admin signup
  console.log('\n[1] Attempting SRP login...');
  const tryLogin = await srpLogin(EMAIL, PASSWORD).catch(e => ({ error: e.message }));
  if (tryLogin && !tryLogin.error) {
    console.log('    Logged in as existing admin.');
    token = tryLogin;
  } else {
    console.log('[1] Login failed, creating admin account...');
    const keys = await generateSrpKeys(EMAIL, PASSWORD);
    const signup = await req('POST', '/api/v1/admin/signup', {
      email: EMAIL, password: PASSWORD,
      firstName: FIRST_NAME, lastName: LAST_NAME,
      ...keys,
    });
    if (signup.status !== 200) {
      console.error('    Admin signup failed:', signup.status, JSON.stringify(signup.body).slice(0, 300));
      process.exit(1);
    }
    token = signup.body.token;
    orgId  = signup.body.organization?.id;
    console.log('    Created. Org ID:', orgId);
  }

  // ── 2. Get orgId if we logged in ──
  if (!orgId) {
    const orgs = await req('GET', '/api/v1/organization', null, token);
    orgId = orgs.body?.organizations?.[0]?.id;
    if (!orgId) {
      const me = await req('GET', '/api/v2/users/me/organizations', null, token);
      orgId = me.body?.organizations?.[0]?.id;
    }
    console.log('[2] Org ID:', orgId);
  }

  // ── 3. Create workspace ──
  console.log('[3] Creating workspace "nyx"...');
  let workspaceId;
  const wsCreate = await req('POST', '/api/v2/workspace', {
    projectName: 'nyx',
    organizationId: orgId,
    shouldCreateDefaultEnvs: true,
  }, token);

  if (wsCreate.status === 200 || wsCreate.status === 201) {
    workspaceId = wsCreate.body?.project?.id || wsCreate.body?.workspace?.id || wsCreate.body?.id;
    console.log('    Created. Workspace ID:', workspaceId);
  } else {
    // Might already exist — list workspaces
    console.log('    Create returned', wsCreate.status, '— checking existing workspaces...');
    const existing = await req('GET', `/api/v1/organization/${orgId}/workspaces`, null, token);
    const ws = existing.body?.workspaces || existing.body?.workspace || [];
    const nyxWs = Array.isArray(ws) ? ws.find(w => w.name === 'nyx' || w.slug === 'nyx') || ws[0] : null;
    workspaceId = nyxWs?.id;
    console.log('    Using existing workspace ID:', workspaceId);
  }

  if (!workspaceId) {
    console.error('    Could not obtain workspace ID');
    process.exit(1);
  }

  // ── 4. Find production environment ──
  console.log('[4] Finding production environment...');
  const envResp = await req('GET', `/api/v1/workspace/${workspaceId}/environments`, null, token);
  const envList = envResp.body?.environments || envResp.body || [];
  const prodEnv = Array.isArray(envList)
    ? (envList.find(e => e.slug === 'prod' || e.name?.toLowerCase() === 'production') || envList[0])
    : null;
  const envSlug = prodEnv?.slug || 'prod';
  console.log('    Using env:', envSlug, '(environments:', envList.map(e => e.slug).join(', '), ')');

  // ── 5. Push secrets ──
  console.log('[5] Pushing secrets to workspace', workspaceId, 'env', envSlug, '...');
  let ok = 0; let fail = 0;
  for (const [key, value] of Object.entries(SECRETS)) {
    // Try creating; if 409 conflict, update
    const createResp = await req('POST', `/api/v3/secrets/raw/${key}`, {
      workspaceId, environment: envSlug,
      secretValue: value, secretComment: '', type: 'shared',
    }, token);

    if (createResp.status === 200 || createResp.status === 201) {
      console.log('  ✓ created', key);
      ok++;
    } else if (createResp.status === 409 || createResp.status === 400) {
      // Already exists — update
      const updateResp = await req('PATCH', `/api/v3/secrets/raw/${key}`, {
        workspaceId, environment: envSlug,
        secretValue: value, type: 'shared',
      }, token);
      if (updateResp.status === 200) {
        console.log('  ↻ updated', key);
        ok++;
      } else {
        console.log('  ✗ failed ', key, updateResp.status, JSON.stringify(updateResp.body).slice(0, 80));
        fail++;
      }
    } else {
      console.log('  ✗ failed ', key, createResp.status, JSON.stringify(createResp.body).slice(0, 80));
      fail++;
    }
  }

  // ── 6. Output ──
  console.log(`\n[6] Done: ${ok} secrets OK, ${fail} failed.`);
  console.log('\n=== .infisical.json content ===');
  console.log(JSON.stringify({ workspaceId, defaultEnvironment: envSlug }, null, 2));
  console.log('==============================\n');
  console.log('Next: infisical login --domain=http://localhost:8090');
  console.log('      Then: just auth-up');
}

main().catch(e => {
  console.error('\nFATAL:', e.message);
  console.error(e.stack?.split('\n').slice(1, 4).join('\n'));
  process.exit(1);
});
