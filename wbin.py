from sys import argv

with open(argv[2], "r") as p:
    with open(argv[1], "bw") as f:
        f.write(bytes(eval(p.read())))