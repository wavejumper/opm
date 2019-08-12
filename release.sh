rm opm.zip
mkdir opm
mkdir -p opm/target/release
cp target/release/helloworld.pd_linux opm/target/release/
cp target/release/libopm.* opm/target/release/ 
cp main.pd opm
zip -r opm.zip opm
rm -rf opm
