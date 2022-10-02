# Does not cover complete function

echo "== TESTING -t TEST DELETE ME PLEASE ==" && \
cargo run -- -t TEST DELETE ME PLEASE && \
echo "== TESTING -t TEST ==" && \
cargo run -- -t TEST && \
echo "== TESTING -t \"TEST\" ==" && \
cargo run -- -t "TEST" && \
echo "== TESTING -p home -t TEST ==" && \
cargo run -- -p home -t TEST && \
echo "== TESTING -p home ==" && \
cargo run -- -p home && \
echo "== TESTING --project home ==" && \
cargo run -- --project home && \
echo "== TESTING -l ==" && \
cargo run -- -l && \
echo "== TESTING --list ==" && \
cargo run -- --list && \
echo "== TESTING -a test 123123 ==" && \
cargo run -- -a test 123123 && \
echo "== TESTING -r test ==" && \
cargo run -- -r test && \
echo "== TESTING --add test 123123 ==" && \
cargo run -- --add test 123123 && \
echo "== TESTING --remove test ==" && \
cargo run -- --remove test && \
echo "== TESTING -s ==" && \
cargo run -- -s && \
echo "== TESTING --sort ==" && \
cargo run -- --sort && \
echo "== TESTING -e ==" && \
cargo run -- -e && \
echo "== TESTING --scheduled ==" && \
cargo run -- --scheduled && \
echo "== TESTING --help ==" && \
cargo run -- --help && \
echo "== TESTING --prioritize ==" && \
cargo run -- --prioritize && \
echo "== TESTING -z ==" && \
cargo run -- -z && \
echo "== TESTING -l -o tests/tod.cfg  ==" && \
cargo run -- -l -o tests/tod.cfg && \
echo "== TESTING -n -p home ==" && \
cargo run -- -n -p home && \
echo ""
echo "== ======= =="
echo "== SUCCESS =="
echo "== ======= =="

