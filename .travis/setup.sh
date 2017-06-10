#!/bin/bash
set -e

cd "$(dirname "$0")"

sudo cp pg_hba.conf $(psql -U postgres -c "SHOW hba_file" -At)

sudo service postgresql stop
sudo service postgresql start 9.6