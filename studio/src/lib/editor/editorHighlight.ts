import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags } from '@lezer/highlight'

function buildHighlightStyle(): HighlightStyle {
  return HighlightStyle.define([
    { tag: tags.keyword, color: 'var(--syntax-keyword)' },
    { tag: tags.string, color: 'var(--syntax-string)' },
    { tag: tags.comment, color: 'var(--syntax-comment)' },
    { tag: tags.number, color: 'var(--syntax-number)' },
    { tag: tags.typeName, color: 'var(--syntax-type)' },
    { tag: tags.propertyName, color: 'var(--syntax-type)' },
    { tag: tags.bool, color: 'var(--syntax-number)' },
    { tag: tags.null, color: 'var(--syntax-number)' },
    { tag: tags.punctuation, color: 'var(--syntax-punctuation)' },
    { tag: tags.separator, color: 'var(--syntax-punctuation)' },
    { tag: tags.operator, color: 'var(--syntax-punctuation)' },
  ])
}

export function createSyntaxHighlightExtension() {
  return syntaxHighlighting(buildHighlightStyle())
}
