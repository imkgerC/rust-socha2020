cargo build -p referee --release
"./target/release/referee.exe" -tc 1000 -n 1000 -t 4 -p1 "./target/release/referee_client.exe" -p2 "./old_versions/referee_client_v7.exe"
pause