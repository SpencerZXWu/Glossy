"""Regenerates the selection fixtures with the exact bytes a real program writes.

The invisible characters these files carry (soft hyphens, zero width spaces,
non breaking spaces, escape sequences) cannot be typed into an editor reliably,
so they are written from here instead.
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "text")

CASES = {
    # A paragraph copied out of a web page: non breaking spaces, zero width
    # spaces at the end of a line, and lines wrapped by the window width.
    "browser": (
        "The quick brown fox\u00a0jumps over the lazy\n"
        "dog. It was the best of times, it was the\n"
        "worst of times.\u200b\u200b\n",
        "The quick brown fox jumps over the lazy dog. It was the best of times, it was the worst of times.\n",
    ),
    # Two paragraphs out of Word: smart quotes, trailing spaces, CRLF and a
    # hard break inside the first sentence.
    "office": (
        "  Smart quotes are placed \u201clike this\u201d and the line\r\n"
        "wraps mid sentence, so the translator\r\n"
        "sees three lines.  \r\n"
        "\r\n"
        "\r\n"
        "A second paragraph ends with a full stop.\r\n"
        "Another sentence follows.\r\n",
        "Smart quotes are placed \u201clike this\u201d and the line wraps mid sentence, so the translator sees three lines.\n"
        "\n"
        "A second paragraph ends with a full stop.\n"
        "Another sentence follows.\n",
    ),
    # A copy out of a terminal: colour codes, a wrapped sentence, a blank line
    # and an indented summary line.
    "terminal": (
        "\u001b[33mcommit\u001b[0m 1a2b3c4 Fix the parser for\n"
        "nested lists\n"
        "\n"
        " 2 files changed, 12 insertions(+)\n",
        "commit 1a2b3c4 Fix the parser for nested lists\n"
        "\n"
        "2 files changed, 12 insertions(+)\n",
    ),
    # A PDF reader hyphenates across the line break.
    "pdf": (
        "This docu-\n"
        "ment was hyphenated by the layout engine.\n"
        "Another sen-\n"
        "tence continues on the next line, too.\n",
        "This document was hyphenated by the layout engine.\n"
        "Another sentence continues on the next line, too.\n",
    ),
    # Chinese has no word separator, so a wrapped line is joined without a
    # space.
    "cjk": (
        "\u8fd9\u662f\u7b2c\u4e00\u53e5\u8bdd\uff0c\n"
        "\u7b2c\u4e8c\u53e5\u8bdd\u7ee7\u7eed\u3002\n",
        "\u8fd9\u662f\u7b2c\u4e00\u53e5\u8bdd\uff0c\u7b2c\u4e8c\u53e5\u8bdd\u7ee7\u7eed\u3002\n",
    ),
}


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, (raw, expected) in CASES.items():
        with open(os.path.join(OUT, name + ".txt"), "w", encoding="utf-8", newline="") as handle:
            handle.write(raw)
        with open(
            os.path.join(OUT, name + ".expected.txt"), "w", encoding="utf-8", newline=""
        ) as handle:
            handle.write(expected)
        print(name, len(raw), len(expected))


if __name__ == "__main__":
    main()
