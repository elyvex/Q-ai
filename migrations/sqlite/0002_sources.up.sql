CREATE TABLE sources (
  id                TEXT PRIMARY KEY,
  title             TEXT NOT NULL,
  alternate_titles  TEXT NOT NULL DEFAULT '[]',
  content_type      TEXT NOT NULL,
  authors           TEXT NOT NULL DEFAULT '[]',
  compiler          TEXT,
  translator        TEXT,
  editor            TEXT,
  publisher         TEXT,
  language          TEXT,
  tradition         TEXT,
  school            TEXT,
  identifiers       TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL
);

CREATE TABLE source_versions (
  id                   TEXT PRIMARY KEY,
  source_id            TEXT NOT NULL REFERENCES sources(id) ON DELETE RESTRICT,
  version              TEXT NOT NULL,
  schema_version       INTEGER NOT NULL,
  state                TEXT NOT NULL CHECK (state IN (
                          'Discovered','PendingReview','Downloading','Downloaded',
                          'Validating','ValidationFailed','Staged','Approved',
                          'Indexing','Active','Deprecated','Quarantined','Removed')),
  trust_level          TEXT NOT NULL CHECK (trust_level IN (
                          'Quarantined','MachineGenerated','UserProvided','ImportedUnverified',
                          'CommunityReviewed','ScholarReviewed','PublisherVerified','CanonicalVerified')),
  license_status       TEXT NOT NULL CHECK (license_status IN (
                          'PublicDomain','OpenLicense','PermissionGranted','UserOwned',
                          'MetadataOnly','Unknown','Restricted')),
  license_json         TEXT NOT NULL,
  manifest_blob_id     TEXT REFERENCES blobs(id),
  manifest_hash        TEXT,
  content_hash         TEXT,
  source_urls          TEXT NOT NULL DEFAULT '[]',
  publication_date     TEXT,
  imported_at          TEXT,
  validated_at         TEXT,
  approved_at          TEXT,
  approved_by          TEXT REFERENCES principals(id),
  activated_at         TEXT,
  deprecated_at        TEXT,
  quarantine_reason    TEXT,
  validation_report    TEXT,
  notes                TEXT,
  created_at           TEXT NOT NULL,
  UNIQUE (source_id, version),
  CHECK (state NOT IN ('Approved','Indexing','Active')
         OR (content_hash IS NOT NULL
             AND license_status <> 'Unknown'
             AND approved_by IS NOT NULL
             AND validation_report IS NOT NULL))
);

CREATE UNIQUE INDEX ux_source_active
  ON source_versions(source_id) WHERE state = 'Active';

CREATE TABLE source_files (
  id                TEXT PRIMARY KEY,
  source_version_id TEXT NOT NULL REFERENCES source_versions(id) ON DELETE CASCADE,
  role              TEXT NOT NULL,
  relative_path     TEXT NOT NULL,
  format            TEXT NOT NULL,
  media_type        TEXT,
  bytes             INTEGER,
  declared_hash     TEXT NOT NULL,
  observed_hash     TEXT,
  encoding          TEXT,
  unicode_form      TEXT CHECK (unicode_form IN ('nfc','nfd','nfkc','nfkd','unknown')),
  blob_id           TEXT REFERENCES blobs(id),
  verified_at       TEXT,
  UNIQUE (source_version_id, relative_path)
);

CREATE TABLE source_genealogy (
  child_source_id    TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  parent_source_id   TEXT NOT NULL REFERENCES sources(id) ON DELETE RESTRICT,
  derivation_type    TEXT NOT NULL CHECK (derivation_type IN
                        ('translation','summary','edition','abridgment','commentary','original')),
  derivation_language TEXT,
  derivation_date    TEXT,
  derivation_note    TEXT,
  PRIMARY KEY (child_source_id, parent_source_id),
  CHECK (child_source_id <> parent_source_id)
);

CREATE TABLE source_state_transitions (
  id                TEXT PRIMARY KEY,
  source_version_id TEXT NOT NULL REFERENCES source_versions(id) ON DELETE CASCADE,
  from_state        TEXT,
  to_state          TEXT NOT NULL,
  actor_id          TEXT REFERENCES principals(id),
  reason            TEXT,
  occurred_at       TEXT NOT NULL
);

CREATE TABLE approvals (
  id                TEXT PRIMARY KEY,
  subject_urn       TEXT NOT NULL,
  kind              TEXT NOT NULL,
  requested_by      TEXT REFERENCES principals(id),
  decided_by        TEXT REFERENCES principals(id),
  decision          TEXT CHECK (decision IN ('approved','denied')),
  request_payload   TEXT NOT NULL,
  decision_note     TEXT,
  requested_at      TEXT NOT NULL,
  decided_at        TEXT
);
