#!/bin/sh

footer="footer.md"

for post in _posts/*.md index.md; do
    cat $footer >> $post
done;
