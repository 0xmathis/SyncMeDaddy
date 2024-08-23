mkdir /tmp/smd 2> /dev/null
rm -rf /tmp/smd/*
cd /tmp/smd

mkdir -p client/storage server/user/storage
mkdir -p client/storage/sub1/sub2
mkdir -p server/user/storage/sub3/sub4


echo test1 > client/storage/test1.txt
echo test2 > server/user/storage/test2.txt
echo test3 > client/storage/sub1/sub2/test3.txt
echo test4 > server/user/storage/sub3/sub4/test4.txt
