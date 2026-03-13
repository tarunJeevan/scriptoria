-- Dev seed: default project for smoke testing
-- This migration seeds a single default project for development purposes.
--
-- TO REVERT: wipe the database, delete this migration, and re-run with only the initial schema migration:
--   rm src-tauri/scriptoria_dev.db src-tauri/migrations/20260101000000_dev_seed_default_project.sql
--   (migrations re-run automatically on next app launch)
--
-- NOTE: This seed will be superseded by the projects management chunk, at which point this file can be deleted and the database wiped.

INSERT INTO projects (id, user_id, title, description, display_order, metadata)
VALUES (1, 1, 'Default Project', 'Default project for development and testing', 0, '{}');
