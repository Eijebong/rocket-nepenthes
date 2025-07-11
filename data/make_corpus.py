import requests
import re


def get_lines_from_novel(url):
    response = requests.get(url)
    if response.status_code == 200:
        txt = []
        for line in response.text.splitlines():
            line = line.strip()
            if line == '----------':
                break
            if line != '':
                words = re.findall(r'\b\w+\b', line)
                txt.extend(words)
    return txt


with open("corpus.txt", "w") as fd:
    url = "http://www.gutenberg.org/files/1342/1342-0.txt"
    fd.write(" ".join(get_lines_from_novel(url)))
