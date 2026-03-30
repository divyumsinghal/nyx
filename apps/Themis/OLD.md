

# Themis

Themis — the housing transparency platform. Same internal structure.

```
Themis/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── listings.rs            # CRUD for property/room listings
│   │   ├── reviews.rs             # Place reviews, landlord reviews
│   │   ├── search.rs              # Geo-search for listings
│   │   ├── inquiries.rs           # Contact listing owner (creates Matrix room)
│   │   ├── profiles.rs            # Public landlord/renter profile
│   │   └── health.rs
│   ├── handlers/
│   ├── services/
│   │   ├── mod.rs
│   │   ├── listing_manager.rs     # Listing lifecycle: create, expire, renew
│   │   ├── review_moderation.rs   # Basic review validation, duplicate detection
│   │   └── geo_search.rs          # PostGIS-backed location search
│   ├── models/
│   │   ├── mod.rs
│   │   ├── listing.rs             # Listing, ListingCreate, ListingResponse
│   │   ├── review.rs              # Review, ReviewCreate
│   │   └── inquiry.rs             # Inquiry (triggers messaging room)
│   ├── queries/
│   └── workers/
│       ├── mod.rs
│       ├── listing_expiry.rs
│       └── search_sync.rs
└── tests/
```

**Themis API surface** (all prefixed with `/api/Themis`):

```
POST   /listings                        # Create listing (multipart: photos + details)
GET    /listings/{id}                   # Single listing
PATCH  /listings/{id}                   # Update own listing
DELETE /listings/{id}                   # Remove own listing
GET    /listings/search                 # Search by location, price, rooms (geo + filters)

POST   /listings/{id}/reviews           # Submit review
GET    /listings/{id}/reviews           # List reviews (cursor-paginated)
GET    /areas/{area_slug}/reviews       # Reviews for an area/neighborhood

POST   /listings/{id}/inquire          # Contact listing owner (creates Matrix DM room)

GET    /profiles/{alias}               # Public profile (landlord or renter)
```
