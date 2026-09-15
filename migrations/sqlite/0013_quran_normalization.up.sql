-- Phase 2 (M1c, P2-T19): normalization rule catalog + profile ladder.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Profiles are append-only: new versions arrive as INSERTs; UPDATE/DELETE
-- are rejected below (violation reports QAI-NORM-0003).

CREATE TABLE normalization_rules (
  rule_id    TEXT NOT NULL,
  version    TEXT NOT NULL,
  kind       TEXT NOT NULL CHECK (kind IN ('deterministic', 'heuristic')),
  description TEXT NOT NULL,
  PRIMARY KEY (rule_id, version)
);

CREATE TABLE normalization_profiles (
  profile_id   TEXT NOT NULL,
  version      TEXT NOT NULL,
  label        TEXT NOT NULL,
  rules_json   TEXT NOT NULL,
  indexed      INTEGER NOT NULL CHECK (indexed IN (0, 1)),
  heuristic    INTEGER NOT NULL CHECK (heuristic IN (0, 1)),
  experimental INTEGER NOT NULL CHECK (experimental IN (0, 1)),
  PRIMARY KEY (profile_id, version)
);

CREATE TRIGGER trg_normalization_rules_no_update
BEFORE UPDATE ON normalization_rules
BEGIN
  SELECT RAISE(ABORT, 'QAI-NORM-0003: normalization catalog is append-only; register a new version');
END;

CREATE TRIGGER trg_normalization_rules_no_delete
BEFORE DELETE ON normalization_rules
BEGIN
  SELECT RAISE(ABORT, 'QAI-NORM-0003: normalization catalog is append-only; register a new version');
END;

CREATE TRIGGER trg_normalization_profiles_no_update
BEFORE UPDATE ON normalization_profiles
BEGIN
  SELECT RAISE(ABORT, 'QAI-NORM-0003: normalization catalog is append-only; register a new version');
END;

CREATE TRIGGER trg_normalization_profiles_no_delete
BEFORE DELETE ON normalization_profiles
BEGIN
  SELECT RAISE(ABORT, 'QAI-NORM-0003: normalization catalog is append-only; register a new version');
END;

INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N01', '1.0.0', 'deterministic', 'Collapse whitespace runs to a single U+0020 and trim ends');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N02', '1.0.0', 'deterministic', 'Remove U+0640 ARABIC TATWEEL');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N03', '1.0.0', 'deterministic', 'Remove harakat and related diacritics (U+064B-U+0652, U+0656-U+0659, U+065A-U+065F)');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N04', '1.0.0', 'deterministic', 'Remove Uthmani annotation marks (U+06D6-U+06ED, U+06DD, U+0615, U+0617-U+061A, U+06E5-U+06E6)');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N05', '1.0.0', 'deterministic', 'Remove U+0670 ARABIC LETTER SUPERSCRIPT ALEF');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N06', '1.0.0', 'deterministic', 'Fold hamza carriers to bare alef/waw/yeh; standalone hamza deleted');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N07', '1.0.0', 'deterministic', 'Fold U+0671 ARABIC LETTER SUPERSCRIPT ALEF WASLA to bare alef');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N08', '1.0.0', 'deterministic', 'Fold U+0649 ALEF MAKSURA to U+064A YEH');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N09', '1.0.0', 'deterministic', 'Fold U+0629 TEH MARBUTA to U+0647 HEH');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N10', '1.0.0', 'deterministic', 'Fold Persian keheh/farsi-yeh/heh variants to Arabic code points');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N11', '1.0.0', 'deterministic', 'Remove zero-width and bidi control characters');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N12', '1.0.0', 'deterministic', 'Remove Arabic and Latin punctuation');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N13', '1.0.0', 'deterministic', 'Fold Arabic-Indic and Extended Arabic-Indic digits to ASCII');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N14', '1.0.0', 'deterministic', 'Remove waqf pause letters (U+06D6-U+06DB)');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N15', '1.0.0', 'deterministic', 'Expand lam-alef presentation ligatures to base sequences');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N16', '1.0.0', 'deterministic', 'Apply Unicode NFC canonical composition');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N17', '1.0.0', 'deterministic', 'Delete all U+0020 spaces');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N18', '1.0.0', 'heuristic', 'Heuristically strip a leading definite article (ال/لل) with ≥2-letter remainder');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N19', '1.0.0', 'heuristic', 'Heuristically strip a leading conjunction (و/ف) with ≥2-letter remainder');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N20', '1.0.0', 'heuristic', 'Heuristically strip a leading preposition (ب/ل/ك) with ≥2-letter remainder');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N21', '1.0.0', 'heuristic', 'Heuristically strip a trailing pronoun with ≥2-char stem');
INSERT INTO normalization_rules (rule_id, version, kind, description) VALUES ('N22', '1.0.0', 'heuristic', 'Experimentally collapse runs of 3+ identical letters to 2');

INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L0.exact', '1.0.0', 'Exact canonical', '[]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L1.ws', '1.0.0', 'Exact after whitespace normalization', '["N01","N11","N16"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L2.marks', '1.0.0', 'Ignore Quranic marks', '["N01","N11","N16","N04","N14"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L3.diacritics', '1.0.0', 'Ignore diacritics', '["N01","N11","N16","N04","N14","N03","N05","N02"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L4.hamza', '1.0.0', 'Normalize hamza/alif', '["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L5.codepoints', '1.0.0', 'Normalize Arabic/Persian code points', '["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08","N10","N13","N09","N15"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L6.skeleton', '1.0.0', 'Space-insensitive skeleton', '["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08","N10","N13","N09","N15","N12","N17"]', 1, 0, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L7.affix', '1.0.0', 'Morphological (heuristic affix) search', '["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08","N10","N13","N09","N15","N18","N19","N20","N21"]', 1, 1, 0);
INSERT INTO normalization_profiles (profile_id, version, label, rules_json, indexed, heuristic, experimental) VALUES ('L8.fuzzy', '1.0.0', 'Fuzzy spelling search', '["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08","N10","N13","N09","N15"]', 0, 0, 1);
