# Thoughts about file formats

I want slide expansion to always be "expand the named template"
and then the input from the markdown to be variables passed in
with the tera context.

In order to simplify matters somewhat, we will define a default
slide template name `harvey-plain`. The full slide content will
always be in `harvey-content` and the speaker notes in `harvey-notes`

# The `harvey-plain` template

```tera
{% extends "harvey-slide.html" %}

{% block content %}
{{harvey-content | safe}}
{% endblock %}

{% block notes %}
{{harvey-notes | safe}}
{% endblock %}

```

# Slide format

I want to have things be semi-close to how remark.js does things,
so I want to use `---` as the slide separator, have any YAML immediately
following the `---` be the metadata. Metadata is separated from slide
content in one of two ways. If the slide marker is `---` then the
separator is a blank line. This is the trivial form. If the slide
marker is longer (four or more dashes) then the YAML must be terminated
by a `...` instead. Note that this is not handled by the YAML parser,
but instead by the top level input splitter.

Anything in the YAML is part of the context passed into the slide expansion
tera invocation, permitting things like classes to be passed down etc.

In remark.js, slides are separated from their speaker notes by `???` which
is not any kind of valid Markdown so we can use it as a plain text splitter.
However there is still the need to have a way to indicate sections of the
markdown (if a slide template has more than one content block)

If the content text contains lines consisting of exactly three stars (ie.
`***`) then those are considered separator lines and split the content into
multiple parts.

Just as the whole slide content (including those separators) will be in
`harvey-content` so will there be a `harvey-contents` array of each part in turn.

An input deck must always start with a slide (ie. `^\-\-\-\-*$`)

Given that slides are separated by lines of dashes, the use of setext headers
is strongly discouraged.

# Slide Metadata

Slide metadata is typically not inherited by subsequent slides, however certain
values are set to inherit by default. Inheritance rules can be changed by the
slide metadata at any time. Unlike the above `harvey-*` names which _can_ be
overridden, there are some slide metadata parts which cannot be overridden for
use beyond their defined use. These are the top level `meta` name and the
dictionary which it must contain. Before parsing _any_ of your slides, Harvey
will parse some slide metadata which consists of:

```yaml
meta:
  content-name: harvey-content
  content-list: harvey-contents
  default-template: harvey-slide
  inherit: [meta template]
  require: []
  deny: []
  ratio: "16:9"
```

The `content-name`, `content-list`, and `default-template` ought to be clear.
The `inherit` list is a set of metadata keys which are inherited from the previous
slide unless overridden. The `require` list is a set of metadata keys which must
be defined for the slide to be valid, and `deny` lists metadata keys which must
not exist. Beyond this, if you want to validate the slide metadata as part of
a linting process, then you should run `harvey metadata` to extract it all and
then lint externally. Finally the `ratio` key indicates the ratio of width to
height we expect the slides to honour. If the viewport does not meet this
ratio then the slideware HTML is expected to scale slides appropriately. This
must be defined as a pair of colon separated numbers (eg. `16:9` or `4:3`)

It is considered an error if `meta` does not appear in `inherit`, or if it
appears in `deny`. The `require` and `deny` checks are performed after any
metadata changes are processed in the slide entry.

```markdown,yaml
---
template: title-page

# Welcome to Harvey
## by Daniel Silverstone

???

* Say hello to everyone
```

# Thoughts about JavaScript and friends

One option to increase the Rustiness of Harvey could be to use web assembly for the
command and control logic; however frankly that's going to be a pain because embedding
that will become huge; and even a "small" webassembly bundle from Rust includes quite
a chunk of JavaScript to launch it.

Instead we shall include a small number of JS files which will govern the functionality.
They can be overridden trivially, however the functionality we want to present includes:

- Navigation forward and backward by slide using arrow keys, clicking, pageup/down etc.
- Opening of a clone (navigation of which is joined to the origin window)
- Always scale slides to be in the same ratio when rendering them. That ratio
  shall be defined by the metadata `ratio` which is passed into the context.
- toggling of a presenter mode
- Blacking out of the display, toggleable
- activation of a printout mode, with and without notes.

The above is in approximate order of importance.

We expect that all of the above can be achieved with fairly simple JavaScript and appropriate
pre-allocated HTML and CSS.

# CSS thoughts

We'll provide some default CSS (well SASS/SCSS) which defines enough of the slideware to get
a default slide deck to build. This is just enough to deal with the JS above and the
slide template `harvey-slide`. Beyond that it is expected that authors will provide
enough CSS/SASS/SCSS to make things work. Everything which is to be set up in the default
slide templates etc. should make use of naming which starts `harvey-` and all styling
will use SASS/SCSS variables whose values can be overridden by the style settings in the
input deck.

# A top level deck file

In order to permit the specification of all sorts of stuff, we support a top level deck
file which is defined as YAML. This deck file lets you set up default slide metadata,
other values to put into the expansion context such as style variable values, etc.
Also it can be used to list the CSS/SCSS/SASS files which should be included into the
CSS bundle inserted into the HTML, as well as the files which contain slides to be
included. Slides are processed in order, so this can be used to arrange multiple
chapters, or to split out certain parts of decks for sharing with others.

An example of a deck file, where anything missing from the file is assumed to be
set to the values herein is:

````yaml
markdown:
  # Classes applied by the various kinds of GFM blockquote tags
  blockquote:
    note: harvey-note
    tip: harvey-tip
    important: harvey-important
    warning: harvey-warning
    caution: harvey-caution
  # Prefix applied to codeblock highlight names
  # eg. if a codeblock starts ```yaml
  code-block-prefix: harvey-highlight-kind-
  # This class is applied to code block rows which are marked for focus
  # If any line in a code block is focussed, the class is also added to the <pre>
  # this permits fading the <pre> by default, and then highlighting the focussed row
  code-block-focus: harvey-highlight-focus
context:
  harvey-minify:
    css: true
    js: true
# See above for default meta block
meta: {}
# A default set of tree sitter rule maps where the key is a CSS class name
# and the value is the tree-sitter-highlight name which matches that.
# This example is incomplete
tree-sitter-highlight:
  harvey-highlight-attribute: attribute
  harvey-highlight-builtin: function.builtin
  harvey-highlight-punct: punctuation
  harvey-highlight-string: string
  harvey-highlight-special-string: string.special
  # ... many more in the default set
styles:
  - harvey-style-toplevel.scss
scripts:
  - harvey-javascript-toplevel.js
template-path:
  - ./harvey-templates
slides: []
````

If you are having difficulty understanding why some CSS isn't working; you can set the
minify rules to false and then minification will be skipped. Of course, if you
choose to override the templates which include the CSS/JS then those context entries
may have no effect.

The style files are parsed by creating a top level include list and then running a single
SASS/SCSS compile on them. This means that you can treat multiple files in a list as being
one file, allowing styles to be split out for sharing across decks with ease.

Finally for those entries which are lists/free-dicts, it is possible to request the defaults
be included. This is particularly useful for `tree-sitter-highlight` if all you want to do
is add a few more specific highlights for your deck. This can be done with a special
entry of the name `default` which takes a boolean value which ought to evaluate to truth.
For lists, you can use the value `default` at some point in the list and that will be
the default values. This permits you to add to the template paths, etc.

The bare minimum deck file would just be the `slides` list.

The supported languages in code blocks is controlled entirely
