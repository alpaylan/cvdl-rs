# CVDL: CV Description Language

![](https://badgen.net/static/license/MIT/blue)
![](https://badgen.net/github/stars/alpaylan/cvdl)

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

## Roadmap and Todo List

- [ ] Add document-level layout
  - [ ] Add document margin
  - [ ] Add document layout(multi-column etc)
  - [ ] Add page cut
- [ ] Add absolute/percentage widths
- [ ] Add CV examples
- [ ] Add layout examples
- [ ] Add logging
- [ ] Standardize u32/f64 usage. Create clear-cut domains for each.
- [ ] Start using data schemas
  - [ ] Add validation
  - [ ] Add serialization
  - [ ] Add deserialization
  - [ ] Add product types
  - [ ] Add custom serializers
- [ ] Add proper font support
  - [ ] Add font-weight
  - [ ] Add font-slope
  - [ ] Add more font types
  - [ ] Properly manage font assets
  - [ ] Check font licensing issues.
- [ ] Add proper testing
  - [ ] I'm still not sure how to do this one.
- [ ] Add URL elements
- [ ] Abstract layout backend better.
  - [ ] Push the document-level details to `document_layout` module. Layout backends should only consume `ElementBox = SpatialBox x Element` and render that.
- [ ] (maybe) Use some Point and Box libraries. Writing our types is okay for now but it's probably better to use something out of the box.
- [ ] Consider the performance effect of `get_width``. If important, switch to binary search instead of linear search.
- [ ] Add proper CLI using Clap.
  - [ ] We should keep a local app directory with some convention that allows us to distinguish files.
  - [ ] We should be able to query the existing layouts/resume files
  - [ ] We should be able to add new layouts
  - [ ] We should be able to add sections to resume files based on some layout
  - [ ] We should provide clear error messages
- [ ] Add TUI for layout management.
  - [ ] Right now, a layout is just an object in a json file and can be edited from there by hand. I want to add a TUI interface for creating/modifying layouts.
- [ ] Implement `with_macro`
  - [ ] We use `with_foo` for structs where we clone the original object and change only one field. The proper way of doing this entails first creating a `modifier object` that expects a `.build()` object at the end so we don't create the same object 5 times. Doing this for each type would take a decent amount of time, so we don't do that for now. If we had a macro though, it would be good to have `#[derive(With)]` and just be able to use this.
- [ ] Add proper debugging mechanisms
  - [ ] Boxes around each element
  - [ ] Boxes around each layout
  - [ ] Lines for margins with width markers
  - [ ] Unique id's to each layout/component/box.
- [ ] Add failover mechanisms
  - [ ] Suggest correction
  - [ ] Use defaults
  - [ ] Partial render
- [ ] Add CONTRIBUTIONS doc.
- [ ] Comment parts of the with custom logic.
- [ ] Document the compilation phases.
- [ ] Add PNG backend.
- [ ] Add ASCII backend.
- [ ] Add ASCII backend.
