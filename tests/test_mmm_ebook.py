import pytest
from faker import Faker

from mmm_ebook import __version__
from mmm_ebook.main import get_rss_url, parse_page, parse_post

fake = Faker()


def test_version():
    assert __version__ == "0.1.0"


def test_get_rss_url():
    url = fake.url()
    assert get_rss_url(url) == url
    assert get_rss_url(url, 2) == f"{url}&paged=2"


def test_parse_page():
    parse_page(1)
    pass


@pytest.mark.skip
def test_parse_post():
    parse_post(1)
    pass
