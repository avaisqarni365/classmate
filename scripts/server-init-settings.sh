#!/bin/bash
sqlite3 /var/lib/classmate/classmate.db <<'SQL'
INSERT OR REPLACE INTO settings (key, value) VALUES ('public_base_url', 'https://cm.codes-ai.uk');
INSERT OR REPLACE INTO settings (key, value) VALUES ('public_hub_path', '/hub');
SQL
