import os
import sys
import subprocess
import math
import argparse

def main(threads, name1='one', name2='two', exec1='./xml_client180.exe', exec2='./xml_client180.exe', tests=10):
    wins = {name1: 0, name2: 0}
    draws = 0
    n = 0
    if tests < threads:
        print("more threads than tests, reducing threads")
        threads = tests
        return
    n_per_thread = int(tests / threads)
    n_over = tests - (n_per_thread*threads)
    n_for_thread = []
    # start server
    print("start the server manually by executing 'java -Dfile.encoding=UTF-8 -Dlogback.configurationFile=logback.xml -jar server.jar --port 13051'")
    print("continue by pressing enter")
    input()
    # server = subprocess.Popen("java -Dfile.encoding=UTF-8 -Dlogback.configurationFile=logback.xml -jar server.jar --port 13051", stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    games = []
    for index in range(threads):
        n_here = n_per_thread if index > 0 else n_per_thread + n_over
        cmd = game_command(name1, name2, exec1, exec2, n_here)
        games.append(subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT))
        n_for_thread.append(n_here)
    while True:
        for (thread_idx, game_runner) in enumerate(games):
            retcode = game_runner.poll()
            line = game_runner.stdout.readline().decode('utf-8')
            # print(line)
            if ('abnormally' in line):
                print(line)
            if ('Game' in line) and ('ended' in line):
                relevant_bit = line.split('Winner')[1]
                n += 1
                n_for_thread[thread_idx] -= 1
                if ':' in relevant_bit:
                    wins[relevant_bit[2:].strip()] += 1
                else:
                    draws += 1
                points_n = (wins[name1] + draws/2) / n
                elo = get_elo_gain(points_n)
                x_a = wins[name1] + draws/2
                k = (1.96 * 1.96 + 2.0 * x_a) / (-1.0 * 1.96 * 1.96 - n)
                q = -1.0 * x_a * x_a / (n * (-1.96 * 1.96 - n))
                root = math.sqrt((k / 2.0) * (k / 2.0) - q)
                p_a_upper = -k / 2.0 + root
                print(f"wld +{wins[name1]} -{wins[name2]} ={draws}, elo {elo} +-{get_elo_gain(p_a_upper)-elo}")
            if (retcode is not None) or ('Exception' in line):
                if n_for_thread[thread_idx] > 0:
                    games[thread_idx] = subprocess.Popen(game_command(name1, name2, exec1, exec2, n_for_thread[thread_idx]), stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                else:
                    all_finished = True
                    for count in n_for_thread:
                        if count > 0:
                            all_finished = False
                    if all_finished:
                        print("Everything is finished, goodbye")
                        return

def get_elo_gain(p_a):
    if p_a > 0.9999999:
        return float("inf")
    try:
        return -1.0 * math.log(1.0 / p_a - 1.0) * 400.0 / math.log(10.0)
    except:
        return float("inf")

def game_command(name1, name2, exec1, exec2, tests):
    return f'java -jar -Dlogback.configurationFile=logback-tests.xml test-client.jar --tests {tests} --name1 "{name1}" --player1 "{exec1}" --name2 "{name2}" --player2 "{exec2}" --port 13051'

if __name__ == "__main__":
    path = os.path.realpath(__file__)
    path = path[: path.rfind("\\")]
    os.chdir(path)
    parser = argparse.ArgumentParser(description='Wrapper for ergonimic use of CAU test client.')
    parser.add_argument('-t', '--threads', help='number of threads to start with, default: 1', default=1, type=int)
    parser.add_argument('-n', '--tests', help='number of tests to execute, default: 1', default=1, type=int)
    parser.add_argument('--exec1', help='path to executable 1', type=str)
    parser.add_argument('--exec2', help='path to executable 1', type=str)
    args = parser.parse_args()
    if args.exec1 is None:
        print("First executable not provided")
        sys.exit()
    if args.exec2 is None:
        print("Second executable not provided")
        sys.exit()
    main(threads=args.threads, tests=args.tests, exec1=args.exec1, exec2=args.exec2)