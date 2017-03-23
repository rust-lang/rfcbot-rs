#!/usr/bin/env bash

set -e

# environment variables
cp /vagrant/vagrant/vagrant_env.sh /etc/profile.d/

# build folder
# cargo shouldn't share items b/t VM and host (using editor build-on-save, for example)
mkdir -p /rust-dashboard/target
chown -R vagrant:vagrant /rust-dashboard/

# dependencies
update-locale LANG=en_US.UTF-8
locale-gen en_US.UTF-8

curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list

apt-get update
apt-get install -y \
    curl \
    git \
    libpq-dev \
    nodejs \
    npm \
    pkg-config \
    postgresql \
    yarn
ln -s /usr/bin/nodejs /usr/bin/node
