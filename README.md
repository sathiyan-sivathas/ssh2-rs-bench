# ssh2-rs-bench
Benchmarking of SSH2 rust libraries

## Set up SSH server

```
# Install - Centos
yum install openssh-server
# Install - Ubuntu
apt install openssh-server

# Create keys
/usr/bin/ssh-keygen -A

# Create autorized keys (use same key for simplicity)
cp /etc/ssh/ssh_host_rsa_key.pub authorized_keys

# Run SSHD
/usr/sbin/sshd
```
