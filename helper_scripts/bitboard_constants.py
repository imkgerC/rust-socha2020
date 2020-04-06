
def bit(x, z):
    return 1 << (x + z*11)

def in_bounds(x, z):
    distance_to_5 = abs(z - 5)
    if z > 5:
        if x > distance_to_5:
            return True
    elif z < 5:
        if x < (10 - distance_to_5):
            return True
    else:
        return True
    return False

def valid_fields():
    valid = 0
    for x in range(11):
        for z in range(11):
            if (in_bounds(x, z)):
                valid |= bit(x, z)
    print("Valid fields")
    print(valid)
    print(bin(valid))

def main():
    valid_fields()

if __name__ == "__main__":
    main()