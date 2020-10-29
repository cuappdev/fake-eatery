cargo build --release
openssl aes-256-cbc -K $encrypted_bfd784009d19_key -iv $encrypted_bfd784009d19_iv -in server.pem.enc -out server.pem -d
ssh -i server.pem -o "StrictHostKeyChecking no" root@64.227.6.156 "systemctl stop fake-eatery"
scp -i server.pem -o "StrictHostKeyChecking no" ./target/release/fake-eatery root@64.227.6.156:/fake-eatery
ssh -i server.pem -o "StrictHostKeyChecking no" root@64.227.6.156 "systemctl start fake-eatery"