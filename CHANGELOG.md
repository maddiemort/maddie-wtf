# Changelog

## [v0.1.4](https://github.com/maddiemort/maddie-wtf/compare/v0.1.3...667665a7c9d6d4cb79677ca59dadeda8eb6614ca) (2025-07-27)

### Features

* make headers into non-highlighted links to their sections
([667665a](https://github.com/maddiemort/maddie-wtf/commit/667665a7c9d6d4cb79677ca59dadeda8eb6614ca))
* move Lobsters/HN links to end of entries, wrap frontmatter/endmatter
properly
([9b320e2](https://github.com/maddiemort/maddie-wtf/commit/9b320e25e98ee373b5ca5140e39725f2922574e2))
* provide GUIDs in RSS feed
([22b20b2](https://github.com/maddiemort/maddie-wtf/commit/22b20b2a9e8bb8356f3a8eecaeb74541d8244641))
* expose metrics, implement first metric `maddie_wtf_requests_received_count`
([e5c28af](https://github.com/maddiemort/maddie-wtf/commit/e5c28af1880c08a276ae7b36174cc546b640bc47))
* allow linking back to Lobsters and HN posts
([51eb638](https://github.com/maddiemort/maddie-wtf/commit/51eb63842f95cc0cfd441ce868cc51c575bafc7d))
* add a link to the RSS feed in the footer
([9a980bb](https://github.com/maddiemort/maddie-wtf/commit/9a980bba21ebdea43de8c4566dfbafcd3d2be737))
* add pubDate to RSS feed items
([9a02764](https://github.com/maddiemort/maddie-wtf/commit/9a027641e8d1a7deb5c782543a474f0552ac2a9e))
* display recent *entries* on index, not whole posts
([edff1a4](https://github.com/maddiemort/maddie-wtf/commit/edff1a4b8d96904dc93c768feae1b39ec147121f))
* replace newlines with spaces in RSS feed
([d221bde](https://github.com/maddiemort/maddie-wtf/commit/d221bde0808d1f8163eede7fa320955b9b794d8e))
* RSS feed generation
([20a1a85](https://github.com/maddiemort/maddie-wtf/commit/20a1a852cba917a3d2833b8b87bc64adf2c222c3))
* split threaded posts more cleanly, give them separate TOCs
([e0d2f12](https://github.com/maddiemort/maddie-wtf/commit/e0d2f123ad50a27414553c97ec0dcdf8fc462275))
* display individual post entries on their own page
([5fde758](https://github.com/maddiemort/maddie-wtf/commit/5fde758d2799cb74781e37e7a08cc8b1502b572d))
* support tables of contents for threaded posts too
([ec8b408](https://github.com/maddiemort/maddie-wtf/commit/ec8b4087c7f7b440fbf11d1b3eaed2bea292f02c))
* allow posts & entries to be manually marked with an update date
([d36fcb7](https://github.com/maddiemort/maddie-wtf/commit/d36fcb7af3bd44e2c75bb3ec29d8a9288694f0db))
* build tables of contents for single-entry posts
([251037c](https://github.com/maddiemort/maddie-wtf/commit/251037c3d68a9ab8a2edbc40e069edc1c130b076))
* use unsafe Markdown rendering, it's my own content
([8504bf8](https://github.com/maddiemort/maddie-wtf/commit/8504bf80190702a3dce527f425add009cc6e8354))
* add styling for aside, increase its and blockquote's padding
([9478559](https://github.com/maddiemort/maddie-wtf/commit/94785593ab978d6a669fd4a0c2c2ccc2e675b009))
* end post summaries at the second heading, or at a cut marker
([517f0da](https://github.com/maddiemort/maddie-wtf/commit/517f0da20a65a669e9bb3934f774a8fed8e93c4e))
* add Read More link below post entries
([47e0288](https://github.com/maddiemort/maddie-wtf/commit/47e02887c223decccd6f78de385d3c4f70e046ec))
* show all posts with a given tag under /tagged/:tag
([9d3d8c9](https://github.com/maddiemort/maddie-wtf/commit/9d3d8c99b1ac46d6449e3c6c6e36a0299ad23c4e))
* add list of tags page at /tags
([2d1cdb0](https://github.com/maddiemort/maddie-wtf/commit/2d1cdb060260e51217df6414316ea4b624a5ed31))
* use zero-padded, not space-padded, day numbers everywhere
([1973981](https://github.com/maddiemort/maddie-wtf/commit/19739817118d45b230e9beb629add648b003c335))
* show list of recent posts on index page
([6e58c19](https://github.com/maddiemort/maddie-wtf/commit/6e58c1980dbdd59045ecaa9c365968c5680433dc))
* add navbar entry for /projects
([ad0a39c](https://github.com/maddiemort/maddie-wtf/commit/ad0a39ce6c0175a5176ebdf363b611b90e169cfb))
* display post tags in a monospace font
([4cbe99f](https://github.com/maddiemort/maddie-wtf/commit/4cbe99fe0d60e581457278d9f4ed536466cd130b))
* always format dates in a human-readable way
([b6fb023](https://github.com/maddiemort/maddie-wtf/commit/b6fb0235d5168c4dd054d18036e64dc770eda4fa))
* activate arbitrary page route handler
([2906f88](https://github.com/maddiemort/maddie-wtf/commit/2906f88e350ff9d5a679a2ff2ade31b880e557ff))
* make page title optional, unify index and page handlers
([26c2f2e](https://github.com/maddiemort/maddie-wtf/commit/26c2f2e7ec3d91bd7829cdfb11c70f9a6144e6a1))
* change separator for horizontal lists to U+2B29 Black Small Diamond
([ca1fb0c](https://github.com/maddiemort/maddie-wtf/commit/ca1fb0ca12ed495f93b29cbe3e7c6fb5ce67fffa))
* use my full name as site title
([a54343a](https://github.com/maddiemort/maddie-wtf/commit/a54343aca028ce2e23da06af2fb68084bc4b580c))
* turn off text justification
([f6019b2](https://github.com/maddiemort/maddie-wtf/commit/f6019b29caae4081afa1b8f22bb79f01b96aa937))

### Fixes

* missing space in footer
([a8473c1](https://github.com/maddiemort/maddie-wtf/commit/a8473c1bfaec00c2bfefbacecdf6d09c5a6e54eb))
* use title of website as title inside RSS feed image tag
([c17dc5b](https://github.com/maddiemort/maddie-wtf/commit/c17dc5b399bb32b80e56f25efad5f386a0c9909c))
* treat markdown titles as pre-escaped in RSS feed
([caff735](https://github.com/maddiemort/maddie-wtf/commit/caff735436a0a96d402871af4cdf74c8c48e38ed))
* use markdown titles in RSS feed, not HTML ones
([99255d5](https://github.com/maddiemort/maddie-wtf/commit/99255d52cc2f6b9d83f853c78cc49bb8494eb700))
* rss isn't supposed to be a void tag
([854f53a](https://github.com/maddiemort/maddie-wtf/commit/854f53a91d98774f3a192184ac1792dab8333535))
* wrap each RSS feed item in an <item> tag
([1bcff85](https://github.com/maddiemort/maddie-wtf/commit/1bcff85a04418865db50e6f52a2768c99968bb69))
* wrap page bodies in a main tag
([51449d7](https://github.com/maddiemort/maddie-wtf/commit/51449d70d5c441127c6fe5d79ce0aba801caa506))
* stop adding padding next to ul in site navbar
([860dd23](https://github.com/maddiemort/maddie-wtf/commit/860dd236fd6928eda7c7bed0e23fe444bb29e7bb))
* avoid extra spacing in special/inline uls, fix nested lists
([a76b66c](https://github.com/maddiemort/maddie-wtf/commit/a76b66ce7426f0ec1d25d72569ca9f11c407a0f9))
* ensure pre always uses monospace font
([9db285e](https://github.com/maddiemort/maddie-wtf/commit/9db285e67639ec91e4f5a35633c846be4e553175))
* make strong display in bold
([a747eb1](https://github.com/maddiemort/maddie-wtf/commit/a747eb17da2eabb4f592ee885b4f6af7ab132489))
* add missed code tag around links to tags
([90ee742](https://github.com/maddiemort/maddie-wtf/commit/90ee7425835d73d291980abb107791c705d8b661))
* link to tags from post entries on /tagged/:tag pages
([7634c60](https://github.com/maddiemort/maddie-wtf/commit/7634c608328f5c41e9e536f88621b81ab6949025))
* display lists with outside style, but add inline padding
([8494ec6](https://github.com/maddiemort/maddie-wtf/commit/8494ec67b980794ba5882717ea942fc24a7b3d28))
* display site header propertly on narrow devices
([819c992](https://github.com/maddiemort/maddie-wtf/commit/819c992c1fb41ae1d7d7b60b6ed411a8a744e906))
* capitalise some missed page headings
([9f6cc0a](https://github.com/maddiemort/maddie-wtf/commit/9f6cc0a4fb20da07f320582126132c77447eda5d))

### [v0.1.3](https://github.com/maddiemort/maddie-wtf/compare/v0.1.2...v0.1.3) (2025-07-19)

#### Fixes

* Nix Rust toolchain missing `version` attribute
([7a8afdc](https://github.com/maddiemort/maddie-wtf/commit/7a8afdcaeacb6df967044547a6e39f8303bf5edd))

### [v0.1.2](https://github.com/maddiemort/maddie-wtf/compare/v0.1.1...v0.1.2) (2025-07-19)

#### Features

* set font to IBM Plex Sans
([466280a](https://github.com/maddiemort/maddie-wtf/commit/466280a863f62efb90692ae6fa88efaf482929aa))
* capitalise page and section titles
([4b322af](https://github.com/maddiemort/maddie-wtf/commit/4b322af0626e2289f26fcf464201efd7011e1ac7))

### [v0.1.1](https://github.com/maddiemort/maddie-wtf/compare/v0.1.0...v0.1.1) (2024-10-20)

#### Features

* respect the "drafts" flag
([b9babb0](https://github.com/maddiemort/maddie-wtf/commit/b9babb038e21af00d097e1e30d00e8a76a6a370d))

## v0.1.0 (2024-10-07)

### Features

* add blockquote styling to stylesheet
([3a845cd](https://github.com/maddiemort/maddie-wtf/commit/3a845cd2336509826544c2f0019509103737d5f7))

### Fixes

* **content-loader:** ignore paths containing hidden files or directories
([2a60800](https://github.com/maddiemort/maddie-wtf/commit/2a6080086938a875f77f236d4b2e30fd91570c0e))
