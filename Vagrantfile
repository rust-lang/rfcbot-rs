# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-16.04"

  config.vm.provision("apt",
                      type: "shell",
                      path: "vagrant/native_deps.sh",
                      keep_color: true)

  config.vm.provision("frontend",
                      type: "shell",
                      path: "vagrant/frontend.sh",
                      keep_color: true)

  config.vm.provision("postgres",
                      type: "shell",
                      path: "vagrant/postgres.sh",
                      keep_color: true,
                      privileged: false,
                      env: { 'PGVERSION' => '9.5', })

  config.vm.provision("rust",
                      type: "shell",
                      path: "vagrant/rust.sh",
                      keep_color: true,
                      privileged: false,
                      env: { 'RUST_NIGHTLY_VERSION' => 'nightly-2016-10-19', },)

  config.vm.network :forwarded_port, guest: 4200, host: 4040
  config.vm.network :forwarded_port, guest: 5432, host: 4050

  config.vm.provider "virtualbox" do |v|
    v.memory = 2048
    v.cpus = 4
  end
end
