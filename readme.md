# pdfs

_pdfs_ is a PDF scripting language. It was born out of frustration with existing PDF tools for simple structural manipulations such as concatenating two PDFs, extracting specific sections, and inserting a blank page every other page[^1].

## Motivation

> You should take this whole section with a grain of ~~salt~~ irony. There is a lot of truth to it, but I don't want to indulge in the idea that I alone have found *the* solution to the problem of PDFs.

PDFs are complicated things. Their myriad intricacies result in equally complicated APIs. The [API I'm using internally](https://github.com/J-F-Liu/lopdf), for instance, provides an example for PDF merging which spans across more than 180 lines of code! Yes, this is true!

This complexity is necessary for any general-purpose PDF tool. Some operations we commonly need to perform, though, are very simple and should not require it. I've found myself vibe-coding entire scripts just to insert a blank page between every non-blank page.

_pdfs_, short for PDF Scripting Language, is intentionally narrow-scoped. It is built to provide an easy way to perform structural manipulations on documents (i.e., adding, removing and rearranging pages, but not modifying their contents). PDFs are treated as lists of pages, which can be extracted and combined to form new documents. The following code, for instance, is used to join two PDFs:

```
left = read('left.pdf');
right = read('right.pdf');
write('output.pdf', left >> right);
```

The best alternative I know of is [iLovePDF](https://www.ilovepdf.com/). It works quite well, but, come on, it is a web app ;) Seriously, though, it fails when you want to perform more complex operations such as reversing the pages or the aforementioned task of inserting a blank page in between every page with content.

## Usage

You can find Linux binaries on the releases page. Once you have installed _pdfs_, run the interpreter on a file of your choice:

```
$ pdfs merge.pdfs
```

## Details

_pdfs_ files have a `.pdfs` extension. _pdfs_ provides two built-in function, `read` and `write`, which are enough to copy a file from one location to another:

```
source = read('test.pdf');
write('result.pdf', source);
```

The `>>` operator is used to concatenate two pdfs:

```
a = read('test_a.pdf');
b = read('test_b.pdf');
result = a >> b;
```

PDFs may be index as if they were an array. This example extracts the first and last pages from a PDF and creates a new one:

```
source = read('test.pdf');
result = [source[0], source[-1]];
```

Finally, ranges can be used to extract whole sections of a PDF using start, (exclusive) end, and step in a [Python-like](https://www.geeksforgeeks.org/python/python-list-slicing/) manner:

```
firstToThird = source[:3];
thirdToLast = source[2:];
all = source[:];
reversed = source[::-1];
odd = source[::2];
even = source[1::2];
```

Finally, slices can be used to extract whole sections of PDFs using a start position, 

[^1]: Yes, there is a need for that. The printers at my university only allow two-sided printing, which is awful when you are trying to print sheet music. The solution I came up with is to insert blank pages every other page.
