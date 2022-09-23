# Aliter
  
This is a rewrite of KaTeX (JS) in Rust.  
This is for the most part a pretty close one-to-one rewrite, which is thankfully possible due to KaTeX being written in a way that isn't too hard to translate to Rust. You can likely compare them side-by-side without much issue.  
Various small changes have been made to fit closer with Rust, or because of confusions of translating them into Rust (ex: some Regex features), and introducing some settings to better control output. However, a current goal is to simply stay mostly similar to the official KaTeX repostory and not to diverge too far so as to make it easy to use any improvements they have.

## Usage 
Overall, this should be able to be used in a similar way to the KaTeX library since Aliter tries to copy their API.  
However, we expose some somewhat internal functions for getting parse trees, because part of the point of this library is to allow rendering in more exotic situations than just html/mathml.  

TODO: Include an actual example.

## Versioning
Currently we do not use stable versions, as we expose internal parts of the API that are explicitly not stable on the KaTeX end of things. As well, there may be existing bugs and reorganizations that would help since this was only recently rewritten.  

## License
  
Obviously, since this is a rewrite of the KaTeX project, we inherit their license (MIT).

TODO: Can we license under APACHE and MIT? I feel like I remember something about APACHE being 'weaker' than MIT, and so it can just be applied on top? Is there actually any benefit to doing this?