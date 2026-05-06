/**
 * Instagram-style multi-step registration:
 *   email → otp → username → done (logged in)
 *
 * The OTP + nyx_id are submitted together to Kratos in the final step.
 */
import { useState, useRef, useEffect } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  TextInput as RNTextInput,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
} from "react-native";
import { useRouter } from "expo-router";
import { TextInput, Button } from "@nyx/ui";
import { useAuth, formatAuthError } from "../../src/context/AuthContext";
import { checkNyxIdAvailability } from "@nyx/api";

type LocalStep = "email" | "otp" | "username";

export default function RegisterScreen() {
  const { startRegistration, verifyOtp, registration } = useAuth();
  const router = useRouter();

  // Local UI step — registration.step drives "email→otp" transition via context;
  // "otp→username" is local only (both fields submitted together to Kratos).
  const [localStep, setLocalStep] = useState<LocalStep>("email");
  const [email, setEmail] = useState("");
  const [otpCode, setOtpCode] = useState("");
  const [nyxId, setNyxId] = useState("");
  const [nyxIdAvailable, setNyxIdAvailable] = useState<boolean | null>(null);
  const [nyxIdChecking, setNyxIdChecking] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const nyxIdCheckTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Sync context step → local step
  useEffect(() => {
    if (registration?.step === "otp" && localStep === "email") {
      setLocalStep("otp");
    }
  }, [registration?.step, localStep]);

  // ── Step 1: Email ──────────────────────────────────────────────────────────

  async function handleSendOtp() {
    const trimmed = email.trim().toLowerCase();
    if (!trimmed || !trimmed.includes("@")) {
      setError("Enter a valid email address.");
      return;
    }
    setError("");
    setLoading(true);
    try {
      await startRegistration(trimmed);
      // useEffect above will advance localStep to "otp"
    } catch (e) {
      setError(formatAuthError(e));
    } finally {
      setLoading(false);
    }
  }

  // ── Step 2: OTP ────────────────────────────────────────────────────────────

  function handleOtpNext() {
    const code = otpCode.trim();
    if (code.length < 6) {
      setError("Enter the 6-digit code we sent you.");
      return;
    }
    setError("");
    setLocalStep("username");
  }

  // ── Step 3: Username + submit ──────────────────────────────────────────────

  function handleNyxIdChange(value: string) {
    // Strip leading @ and spaces; lowercase
    const clean = value.replace(/^@/, "").replace(/\s/g, "").toLowerCase();
    setNyxId(clean);
    setNyxIdAvailable(null);

    if (nyxIdCheckTimer.current) clearTimeout(nyxIdCheckTimer.current);
    if (clean.length < 3) return;

    nyxIdCheckTimer.current = setTimeout(async () => {
      setNyxIdChecking(true);
      try {
        const result = await checkNyxIdAvailability(clean);
        setNyxIdAvailable(result.available);
      } catch {
        setNyxIdAvailable(null);
      } finally {
        setNyxIdChecking(false);
      }
    }, 500);
  }

  async function handleCompleteRegistration() {
    if (!nyxId || nyxId.length < 3) {
      setError("Username must be at least 3 characters.");
      return;
    }
    if (nyxIdAvailable === false) {
      setError("That username is taken. Choose another.");
      return;
    }
    setError("");
    setLoading(true);
    try {
      await verifyOtp(otpCode.trim(), nyxId);
      // AuthProvider saves session + clears registration → root nav sends to "/"
    } catch (e) {
      setError(formatAuthError(e));
      // If OTP was wrong, go back to OTP step
      setLocalStep("otp");
      setOtpCode("");
    } finally {
      setLoading(false);
    }
  }

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === "ios" ? "padding" : "height"}
      className="flex-1"
    >
      <View className="flex-1 items-center justify-center p-6">
        {/* Logo */}
        <View className="items-center mb-10">
          <Text
            className="text-5xl font-bold mb-2"
            style={{ color: "#FFD93D", letterSpacing: 2 }}
          >
            nyx
          </Text>
        </View>

        {/* Card */}
        <View className="w-full max-w-xs">
          {/* Error banner */}
          {error ? (
            <View className="bg-red-500/10 border border-red-500/20 rounded-xl p-3 mb-4">
              <Text className="text-red-400 text-sm text-center">{error}</Text>
            </View>
          ) : null}

          {/* ── Step 1: Email ── */}
          {localStep === "email" && (
            <>
              <Text className="text-star-100 text-xl font-semibold text-center mb-1">
                Create account
              </Text>
              <Text className="text-star-400 text-sm text-center mb-6">
                Enter your email to get started
              </Text>

              <TextInput
                placeholder="Email address"
                value={email}
                onChangeText={setEmail}
                autoCapitalize="none"
                keyboardType="email-address"
                autoComplete="email"
                returnKeyType="next"
                onSubmitEditing={handleSendOtp}
                variant="minimal"
                className="mb-6"
              />

              <Button
                label="Continue"
                variant="dawn"
                size="lg"
                fullWidth
                loading={loading}
                onPress={handleSendOtp}
              />

              <View className="flex-row justify-center mt-4">
                <Text className="text-star-400 text-sm">
                  Already have an account?{" "}
                </Text>
                <TouchableOpacity onPress={() => router.replace("/login")}>
                  <Text className="text-dawn-400 text-sm font-semibold">
                    Log in
                  </Text>
                </TouchableOpacity>
              </View>
            </>
          )}

          {/* ── Step 2: OTP ── */}
          {localStep === "otp" && (
            <>
              <Text className="text-star-100 text-xl font-semibold text-center mb-1">
                Enter the code
              </Text>
              <Text className="text-star-400 text-sm text-center mb-6">
                We sent a 6-digit code to{"\n"}
                <Text className="text-star-200 font-medium">{email}</Text>
              </Text>

              {/* Centered OTP input */}
              <RNTextInput
                value={otpCode}
                onChangeText={(v) => setOtpCode(v.replace(/[^0-9]/g, "").slice(0, 6))}
                keyboardType="number-pad"
                maxLength={6}
                textAlign="center"
                returnKeyType="next"
                onSubmitEditing={handleOtpNext}
                placeholder="••••••"
                placeholderTextColor="#4A3F6B"
                style={{
                  fontSize: 28,
                  letterSpacing: 16,
                  color: "#F0EBF8",
                  borderBottomWidth: 2,
                  borderBottomColor: "#FFD93D",
                  paddingVertical: 12,
                  marginBottom: 24,
                  outline: "none",
                } as never}
              />

              <Button
                label="Continue"
                variant="dawn"
                size="lg"
                fullWidth
                onPress={handleOtpNext}
              />

              <TouchableOpacity
                className="items-center mt-4"
                onPress={() => {
                  setLocalStep("email");
                  setOtpCode("");
                  setError("");
                }}
              >
                <Text className="text-star-400 text-sm">
                  Wrong email?{" "}
                  <Text className="text-dawn-400 font-semibold">Change it</Text>
                </Text>
              </TouchableOpacity>
            </>
          )}

          {/* ── Step 3: Username ── */}
          {localStep === "username" && (
            <>
              <Text className="text-star-100 text-xl font-semibold text-center mb-1">
                Choose your username
              </Text>
              <Text className="text-star-400 text-sm text-center mb-6">
                Pick a unique name — you can change it later
              </Text>

              <View className="mb-1">
                <TextInput
                  placeholder="@username"
                  value={nyxId ? `@${nyxId}` : ""}
                  onChangeText={handleNyxIdChange}
                  autoCapitalize="none"
                  autoCorrect={false}
                  returnKeyType="done"
                  onSubmitEditing={handleCompleteRegistration}
                  variant="minimal"
                />
              </View>

              {/* Availability feedback */}
              <View className="h-5 mb-4">
                {nyxIdChecking && (
                  <Text className="text-star-400 text-xs">Checking…</Text>
                )}
                {!nyxIdChecking && nyxIdAvailable === true && nyxId.length >= 3 && (
                  <Text className="text-green-400 text-xs">
                    @{nyxId} is available
                  </Text>
                )}
                {!nyxIdChecking && nyxIdAvailable === false && (
                  <Text className="text-red-400 text-xs">
                    @{nyxId} is taken
                  </Text>
                )}
              </View>

              <Button
                label="Create account"
                variant="dawn"
                size="lg"
                fullWidth
                loading={loading}
                disabled={nyxIdAvailable === false}
                onPress={handleCompleteRegistration}
              />

              <TouchableOpacity
                className="items-center mt-4"
                onPress={() => {
                  setLocalStep("otp");
                  setError("");
                }}
              >
                <Text className="text-star-400 text-sm">
                  Back
                </Text>
              </TouchableOpacity>
            </>
          )}
        </View>

        {/* Progress dots */}
        <View className="flex-row gap-2 mt-10">
          {(["email", "otp", "username"] as LocalStep[]).map((s) => (
            <View
              key={s}
              className={`w-2 h-2 rounded-full ${
                localStep === s ? "bg-dawn-400" : "bg-space-600"
              }`}
            />
          ))}
        </View>
      </View>
    </KeyboardAvoidingView>
  );
}
