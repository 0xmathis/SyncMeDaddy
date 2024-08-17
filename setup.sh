rm -rf /tmp/smd

cd /tmp
mkdir smd

cd smd
mkdir -p client/storage server/user/storage

cd client/storage
echo test1 > test1.txt
