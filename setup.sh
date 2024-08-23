mkdir /tmp/smd 2> /dev/null
rm -rf /tmp/smd/*
cd /tmp/smd

mkdir -p client/storage server/user/storage

echo test1 > client/storage/test1.txt
echo test2 > server/user/storage/test2.txt
