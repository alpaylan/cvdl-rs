# CVDL: CV Description Language

CVDL is an open-source document model focused on creating templates for structured documents. For the short term,
our focus is on CV's, in the long term, we will widen the constructs in the language for supporting more document types easily.

A CVDL document is composed of three parts.

- A set of data schemas, defining the datatypes of sections
- A set of layout schemas, defining the stylistic aspects of the sections
- A document with a set of sections, where each section defines the data and layout schema it conforms to.

For examples of these files, you can see `data/data-schemas.json`, `data/layout-schemas.json`, `data/resume.json`. `data/resume2.json`. The example output right now is on `results/output.pdf`.

If you want to run the executable, you can run using the scheme below. You can change the files for playing around
with your own CV.

`cargo run -- data/data-schemas.json data/layout-schemas.json data/resume2.json results/output.pdf`
