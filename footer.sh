#!/bin/sh

footer="footer.md"

for post in posts/*.md; do
    cat $footer >> $post
done;
