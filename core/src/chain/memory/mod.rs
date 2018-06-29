use std;

use chain::entry::Entry;
use chain::pair::Pair;
use chain::chain::SourceChain;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MemChain {
    pairs: Vec<Pair>,
    top: Option<Pair>,
}

impl MemChain {
    pub fn new() -> MemChain {
        MemChain {
            pairs: Vec::new(),
            top: None,
        }
    }
}

// for loop support that consumes chains
impl IntoIterator for MemChain {
    type Item = Pair;
    type IntoIter = std::vec::IntoIter<Pair>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

// iter() style support for references to chains
impl<'a> IntoIterator for &'a MemChain {
    type Item = &'a Pair;
    type IntoIter = std::slice::Iter<'a, Pair>;

    fn into_iter(self) -> std::slice::Iter<'a, Pair> {
        self.pairs.iter()
    }
}

// basic SouceChain trait
impl<'de> SourceChain<'de> for MemChain {

    // appends the current pair to the top of the chain
    fn push(&mut self, entry_type: String, entry: &Entry) -> Pair {

        let pair = Pair::new(self, entry_type, entry);

        if !(pair.validate()) {
            panic!("attempted to push an invalid pair for this source chain");
        }

        let top_pair = self.top();
        let next_pair = pair.header().next().and_then(|h| self.get(h));
        if !(top_pair == next_pair) {
            // we panic because no code path should attempt to append an invalid pair
            panic!("top pair did not match next hash pair from pushed pair: {:?} vs. {:?}", top_pair, next_pair);
        }

        // dry run an insertion against a clone and validate the outcome
        let mut validation_chain = self.clone();
        validation_chain.top = Some(pair.clone());
        validation_chain.pairs.insert(0, pair.clone());
        if !validation_chain.validate() {
            // we panic because no code path should ever invalidate the chain
            panic!("adding this pair would invalidate the source chain");
        }

        // @TODO - inserting at the start of a vector is O(n), some other collection could be O(1)
        // @see https://github.com/holochain/holochain-rust/issues/35
        self.top = Some(pair.clone());
        self.pairs.insert(0, pair.clone());

        pair
    }

    fn iter(&self) -> std::slice::Iter<Pair> {
        self.pairs.iter()
    }

    fn validate(&self) -> bool {
        self.pairs.iter().all(|p| p.validate())
    }

    fn get(&self, header_hash: u64) -> Option<Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.header().hash() == header_hash)
    }

    fn get_entry(&self, entry_hash: u64) -> Option<Pair> {
        // @TODO - this is a slow way to do a lookup
        // @see https://github.com/holochain/holochain-rust/issues/50
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.entry().hash() == entry_hash)
    }

    fn top(&self) -> Option<Pair> {
        self.top.clone()
    }

    fn top_type(&self, t: &str) -> Option<Pair> {
        self.pairs
            .clone()
            .into_iter()
            .find(|p| p.header().entry_type() == t)
    }

}

#[cfg(test)]
mod tests {
    use serde_json;
    use chain::entry::Entry;
    use chain::pair::Pair;
    use chain::chain::SourceChain;

    #[test]
    fn validate() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        // for valid pairs its truetles all the way down...
        assert!(chain.validate());
        chain.push(entry_type.clone(), &e1);
        assert!(chain.validate());
        chain.push(entry_type.clone(), &e2);
        assert!(chain.validate());
        chain.push(entry_type.clone(), &e3);
        assert!(chain.validate());
    }

    #[test]
    fn get() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        let p1 = chain.push(entry_type.clone(), &e1);
        let p2 = chain.push(entry_type.clone(), &e2);
        let p3 = chain.push(entry_type.clone(), &e3);

        assert_eq!(None, chain.get(0));
        assert_eq!(Some(p1.clone()), chain.get(p1.header().hash()));
        assert_eq!(Some(p2.clone()), chain.get(p2.header().hash()));
        assert_eq!(Some(p3.clone()), chain.get(p3.header().hash()));
    }

    #[test]
    fn get_entry() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        let p1 = chain.push(entry_type.clone(), &e1);
        let p2 = chain.push(entry_type.clone(), &e2);
        let p3 = chain.push(entry_type.clone(), &e3);

        assert_eq!(None, chain.get(0));
        assert_eq!(Some(p1.clone()), chain.get_entry(p1.entry().hash()));
        assert_eq!(Some(p2.clone()), chain.get_entry(p2.entry().hash()));
        assert_eq!(Some(p3.clone()), chain.get_entry(p3.entry().hash()));
    }

    #[test]
    fn valid_push() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        let p1 = chain.push(entry_type.clone(), &e1);
        let p2 = chain.push(entry_type.clone(), &e2);
        let p3 = chain.push(entry_type.clone(), &e3);

        assert_eq!(p1.entry(), e1);
        assert_eq!(p2.entry(), e2);
        assert_eq!(p3.entry(), e3);
    }

    #[test]
    fn iter() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"foo".to_string());

        let p1 = chain.push(entry_type.clone(), &e1);
        let p2 = chain.push(entry_type.clone(), &e2);
        let p3 = chain.push(entry_type.clone(), &e3);

        // iter() should iterate over references
        assert_eq!(vec![&p3, &p2, &p1], chain.iter().collect::<Vec<&Pair>>());

        // iter() should support functional logic
        assert_eq!(
            vec![&p3, &p1],
            chain
                .iter()
                .filter(|p| p.entry().content() == "foo")
                .collect::<Vec<&Pair>>()
        );
    }

    #[test]
    fn into_iter() {
        let mut chain = super::MemChain::new();

        let entry_type = "fooType".to_string();

        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        let p1 = chain.push(entry_type.clone(), &e1);
        let p2 = chain.push(entry_type.clone(), &e2);
        let p3 = chain.push(entry_type.clone(), &e3);

        // into_iter() by reference
        let mut i = 0;
        let expected = [&p3, &p2, &p1];
        for p in &chain {
            assert_eq!(expected[i], p);
            i = i + 1;
        }

        // do functional things with (&chain).into_iter()
        assert_eq!(
            vec![&p1],
            (&chain)
                .into_iter()
                .filter(|p| p.header().next() == None)
                .collect::<Vec<&Pair>>()
        );

        // into_iter() move
        let mut i = 0;
        let expected = [p3.clone(), p2.clone(), p1.clone()];
        for p in chain.clone() {
            assert_eq!(expected[i], p);
            i = i + 1;
        }
    }

    #[test]
    fn json_round_trip() {
        let mut chain = super::MemChain::new();

        let entry_type = "foo".to_string();
        let e1 = Entry::new(&"foo".to_string());
        let e2 = Entry::new(&"bar".to_string());
        let e3 = Entry::new(&"baz".to_string());

        chain.push(entry_type.clone(), &e1);
        chain.push(entry_type.clone(), &e2);
        chain.push(entry_type.clone(), &e3);

        let json = serde_json::to_string(&chain).unwrap();
        let expected_json = "{\"pairs\":[{\"header\":{\"Type\":\"foo\",\"Time\":\"\",\"HeaderLink\":3223843486057940362,\"EntryLink\":16260972211344176173,\"TypeLink\":3223843486057940362,\"Signature\":\"\"},\"entry\":{\"content\":\"baz\",\"hash\":16260972211344176173}},{\"header\":{\"Type\":\"foo\",\"Time\":\"\",\"HeaderLink\":14176581647729525889,\"EntryLink\":3676438629107045207,\"TypeLink\":14176581647729525889,\"Signature\":\"\"},\"entry\":{\"content\":\"bar\",\"hash\":3676438629107045207}},{\"header\":{\"Type\":\"foo\",\"Time\":\"\",\"HeaderLink\":null,\"EntryLink\":4506850079084802999,\"TypeLink\":null,\"Signature\":\"\"},\"entry\":{\"content\":\"foo\",\"hash\":4506850079084802999}}],\"top\":{\"header\":{\"Type\":\"foo\",\"Time\":\"\",\"HeaderLink\":3223843486057940362,\"EntryLink\":16260972211344176173,\"TypeLink\":3223843486057940362,\"Signature\":\"\"},\"entry\":{\"content\":\"baz\",\"hash\":16260972211344176173}}}";

        assert_eq!(expected_json, json);
        assert_eq!(chain, serde_json::from_str(&json).unwrap());
    }

}