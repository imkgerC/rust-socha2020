cargo build -p referee --release
"./target/release/referee.exe" -n 1000 -t 4 -p1 "./target/release/referee_client.exe" -p2 "test.exe"
pause