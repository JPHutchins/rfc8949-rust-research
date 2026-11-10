"""When do RFC 7049 canonical (length-first) and RFC 8949 core-deterministic (bytewise) map-key orders diverge?"""
import random
import string
from collections.abc import Callable
from itertools import product


def head(major: int, argument: int) -> bytes:
    match argument:
        case n if n < 24:
            return bytes([major << 5 | n])
        case n if n < 0x100:
            return bytes([major << 5 | 24, n])
        case n if n < 0x10000:
            return bytes([major << 5 | 25]) + n.to_bytes(2, "big")
        case n if n < 0x100000000:
            return bytes([major << 5 | 26]) + n.to_bytes(4, "big")
        case n:
            return bytes([major << 5 | 27]) + n.to_bytes(8, "big")


def encode_uint(n: int) -> bytes:
    return head(0, n)


def encode_nint(n: int) -> bytes:
    return head(1, -1 - n)


def encode_text(s: str) -> bytes:
    return head(3, len(s.encode())) + s.encode()


def bytewise(keys: list[bytes]) -> list[bytes]:
    return sorted(keys)


def length_first(keys: list[bytes]) -> list[bytes]:
    return sorted(keys, key=lambda k: (len(k), k))


def random_text(rng: random.Random) -> bytes:
    return encode_text("".join(rng.choices(string.ascii_lowercase, k=rng.choice([0, 1, 2, 3, 5, 22, 23, 24, 25, 255, 256, 300]))))


def random_uint(rng: random.Random) -> bytes:
    return encode_uint(rng.choice([0, 23, 24, 255, 256, 65535, 65536, 2**32, rng.randrange(2**40)]))


def random_nint(rng: random.Random) -> bytes:
    return encode_nint(-rng.choice([1, 24, 25, 256, 257, 65536, 65537, rng.randrange(1, 2**40)]))


def divergence_rate(generators: list[Callable[[random.Random], bytes]], trials: int = 20000) -> float:
    rng = random.Random(7049)
    return sum(
        bytewise(keys) != length_first(keys)
        for keys in ([rng.choice(generators)(rng) for _ in range(6)] for _ in range(trials))
    ) / trials


if __name__ == "__main__":
    cases = {
        "text keys only": [random_text],
        "uint keys only": [random_uint],
        "nint keys only": [random_nint],
        "uint + nint (COSE-style labels)": [random_uint, random_nint],
        "uint + text": [random_uint, random_text],
    }
    for name, generators in cases.items():
        print(f"{name}: divergence in {divergence_rate(generators):.1%} of random 6-key maps")
    print("RFC 8949 §4.2.1 example, bytewise:", [k.hex() for k in bytewise([encode_uint(10), encode_uint(100), encode_nint(-1), encode_text("z"), encode_text("aa"), bytes.fromhex("811864"), bytes.fromhex("8120"), bytes.fromhex("f4")])])
    print("RFC 8949 §4.2.3 example, length-first:", [k.hex() for k in length_first([encode_uint(10), encode_uint(100), encode_nint(-1), encode_text("z"), encode_text("aa"), bytes.fromhex("811864"), bytes.fromhex("8120"), bytes.fromhex("f4")])])
