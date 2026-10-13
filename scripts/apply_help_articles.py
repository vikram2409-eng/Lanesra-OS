#!/usr/bin/env python3
ARTICLES = open('scripts/help_articles_body.txt', encoding='utf-8').read()
def patch(path):
    src = open(path, encoding='utf-8').read()
    marker = "   {slug:'custom-fields',title:'Custom Fields:"
    assert src.count(marker) == 1, "marker not found"
    src = src.replace(marker, ARTICLES + "\n" + marker)
    open(path, 'w', encoding='utf-8').write(src)
    print("articles injected")
if __name__ == '__main__':
    patch('app.js')
