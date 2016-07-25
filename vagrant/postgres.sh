#!/usr/bin/env bash

set -e

# setup postgres
sudo -u postgres pg_dropcluster --stop $PGVERSION main
sudo -u postgres pg_createcluster --locale en_US.UTF-8 -e UTF8 --start $PGVERSION main

sudo -u postgres createuser -s --createdb vagrant
createdb -E UTF8 -l en_US.UTF8 -T template0 -O vagrant dashboard
sudo -u postgres echo "ALTER ROLE vagrant WITH PASSWORD 'foobar'" | psql -d dashboard
psql -d dashboard -f /vagrant/bootstrap.sql

sudo cp /vagrant/vagrant/pg_hba.conf /etc/postgresql/9.5/main/
echo "listen_addresses = '*'" | sudo tee --append /etc/postgresql/$PGVERSION/main/postgresql.conf
sudo systemctl restart postgresql
