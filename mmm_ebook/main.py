import os
from functools import partial
from pprint import pprint
from typing import Any, Callable, Iterator

import requests
from bs4 import BeautifulSoup

RSS_URL = "http://www.mrmoneymustache.com/feed/?order=ASC"

# Types
ParsePostResult = dict[str, str]
ParsePageResult = list[Any]

BOOK_DATA = os.path.join(
    os.path.dirname(__file__),
    "import_index.html_in_this_folder_in_calibre_to_create_ebook",
)


def get_rss_url(url: str, page_no: int = None) -> str:
    return url if page_no is None else f"{url}&paged={page_no}"


def parse_post(post: Any) -> ParsePostResult:
    return {
        "title": post.find("title").text,
        "link": post.find("link").text,
        "pubDate": post.find("pubDate").text,
        "content": post.find("encoded").text,
    }


def parse_page(page_number: int) -> ParsePageResult:
    url = get_rss_url(RSS_URL, page_number)
    page = requests.get(url)
    soup = BeautifulSoup(page.content, features="xml")
    print(type(soup.findAll("item")))

    return [parse_post(item) for item in soup.findAll("item")]


def paginator(
    parse_page_func: Callable[[int], ParsePageResult], page_number: int = 1
) -> Iterator[ParsePageResult]:
    done = False

    while not done:
        res = parse_page_func(page_number)
        if len(res):
            yield res
            page_number += 1
        else:
            done = True


def main() -> None:
    try:
        p = paginator(partial(parse_page))
        while True:
            try:
                res = next(p)
                pprint(res)
            except StopIteration:
                break

    except Exception as e:
        print(f"Error parsing RSS feed: {type(e)}")


if __name__ == "__main__":
    main()
