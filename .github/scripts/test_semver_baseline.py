import io
import json
import unittest
from urllib.error import HTTPError, URLError

from semver_baseline import published_baseline


class PublishedBaselineTests(unittest.TestCase):
    def resolve(self, versions):
        return published_baseline("spaceui", lambda *args, **kwargs: io.StringIO(json.dumps({"versions": versions})))

    def test_selects_numeric_latest_non_yanked_stable(self):
        self.assertEqual(self.resolve([
            {"num": "0.1.9", "yanked": False},
            {"num": "0.1.10", "yanked": False},
            {"num": "0.2.0", "yanked": True},
            {"num": "0.3.0-alpha.1", "yanked": False},
        ]), "0.1.10")

    def test_only_404_means_first_publication(self):
        def not_found(*args, **kwargs):
            raise HTTPError("https://crates.io", 404, "not found", {}, None)
        self.assertIsNone(published_baseline("spaceui", not_found))
        for failure in [HTTPError("https://crates.io", 503, "unavailable", {}, None), URLError("offline")]:
            def unavailable(*args, **kwargs):
                raise failure
            with self.assertRaises(type(failure)):
                published_baseline("spaceui", unavailable)

    def test_empty_or_all_yanked_response_is_an_error(self):
        for versions in [[], [{"num": "0.1.0", "yanked": True}]]:
            with self.assertRaises(ValueError):
                self.resolve(versions)

    def test_prerelease_only_crate_gets_a_real_baseline(self):
        self.assertEqual(self.resolve([
            {"num": "0.1.0-alpha.1", "yanked": False, "created_at": "2026-01-01"},
            {"num": "0.1.0-alpha.2", "yanked": False, "created_at": "2026-02-01"},
        ]), "0.1.0-alpha.2")


if __name__ == "__main__":
    unittest.main()
