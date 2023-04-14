# batyr

An XML-to-Postscript converter for creating typed screenplays on a
home or office printer

Produces typewriter output in standard screenplay format.  In the
film industry today, proprietary software is the norm.  For those
of us who are not in the industry, proprietary software is too
expensive, and it will put our work in jeopardy one day when we
need to foot the bill for an upgrade we don't want.

Nothing can beat plain text for the long-term security of the
words we write, but formatting is crucial for screenplays.  Enter
XML, which allows us to add semantics in text form.  We don't need
much.  The screenplay format is actually simpler than it looks;
all of the finesse is in breaking the pages correctly.

That said, Batyr is kind of stupid, relatively speaking.  It does
not really "know" the format; _you_ need to know the format.  But
if you do know what you want to see on the page, you should find
that it is fairly easy to express using the XML schema provided.

The focus here is on the mostly unadorned spec script, but there
is minimal support for scene numbers because it's fun, and because
having a different look can sometimes help with visualization.
The target audience for this software is writers who are looking
for a robust modern way to produce old-fashioned typed hard copy.

This crate is named after an elephant from Kazakhstan who
addressed the Soviet Union on state television in 1980.

Pronunciation (IPA): ,bɑ‘tir

## Examples

Processing a valid document, an encoding of the shooting script
for _It's a Wonderful Life_, by Frances Goodrich, Albert Hackett,
Frank Capra and Jo Swerling:

```sh
$ head -4 goodrich.tyr
<?xml version="1.0" encoding="utf-8"?>
<screenplay
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:noNamespaceSchemaLocation="http://www.matchlock.com/batyr/screenplay.xsd">
$ wc -l goodrich.tyr
9886 goodrich.tyr
$ xmllint --noout --schema screenplay.xsd goodrich.tyr
goodrich.tyr validates
$ batyr goodrich.tyr > goodrich.ps
$ head -6 goodrich.ps
%!PS
%%Title: IT'S A WONDERFUL LIFE
%%Creator: batyr
%%DocumentFonts: Courier
%%BoundingBox: 0 0 612 792
%%Pages: 198
$
```
Output: [`goodrich.pdf`]

Batyr can also show you its internal element representation using
the <tt>-e</tt> flag.  The internal element representation will
also be printed if the input file contains a fragment of the
screenplay schema:

```sh
$ cat minimal.tyr
<br/>
$ batyr minimal.tyr
Br(EmptyElement { attributes: Br, break_info: Disposable(1) })
$
```

## References
<ol>
  <li>Christopher Riley, <em>The Hollywood Standard: The Complete
  and Authoritative Guide to Script Format & Style</em>, 3rd
  edition (Studio City, CA: Michael Wiese Productions, 2021).</li>
  <li>Hillis R. Cole, Jr. and Judith H. Haag, <em>The Complete
  Guide to Standard Script Formats</em> (North Hollywood, CA: CMC
  Publishing, 1996).</li>
</ol>

[`goodrich.pdf`]: <http://www.matchlock.com/batyr/goodrich.pdf>
