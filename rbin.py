from sys import argv

with open(argv[1], "rb") as f:
    print(f.read().hex())