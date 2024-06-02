
use std::collections::{BinaryHeap, HashMap};

use serde::{Serialize, Deserialize};
use postcard;

#[derive(Eq, PartialEq, PartialOrd, Ord,
         Clone, Debug,
         Serialize, Deserialize)]
enum HuffmanTree {
    Leaf(char),
    Node((Box<HuffmanTree>, Box<HuffmanTree>))
}

#[derive(Serialize, Deserialize)]
pub struct SerialisedHuffmanTree {
    tree: HuffmanTree,
    encoded_chars: Vec<u8>
}

type Frequencies = [usize; 256];

#[derive(Eq, PartialEq, PartialOrd, Debug)]
pub struct HuffmanFreqTree {
    frequencies: Frequencies,
    tree: HuffmanTree
}

#[derive(Debug)]
pub struct Huffman {
    freq_tree: HuffmanFreqTree,
    text: String
}

#[derive(Debug, Clone)]
enum Sense {
    Left,
    Right
}

type Path = Vec<Sense>;
type Codewords = HashMap<char, Path>;

impl HuffmanTree {
    pub fn weight(self: &Self, frequencies: Frequencies) -> usize {
         match self {
            HuffmanTree::Leaf(c) => {
                let n = *c as usize;
                frequencies[n] * n
            },
            HuffmanTree::Node((s, t)) =>
                s.weight(frequencies) + t.weight(frequencies)
         }
    }

    fn fill_codewords_with_acc(self: &Self, codewords: &mut Codewords, current_path: Path) {
        match self {
            HuffmanTree::Leaf(c) => {
                let _ = codewords.insert(*c, current_path);
            }
            HuffmanTree::Node((s, t)) => {
                let mut left_dir = current_path.clone();
                let mut right_dir = current_path.clone();
                left_dir.push(Sense::Left);
                right_dir.push(Sense::Right);
                s.fill_codewords_with_acc(codewords, left_dir);
                t.fill_codewords_with_acc(codewords, right_dir);
            }
        }
    }
}

impl SerialisedHuffmanTree {
    pub fn serialise(self: &Self, filepath: String) -> Result<String, String> {
        let serial = postcard::to_allocvec(&self)
            .expect("A valid serialisation");

        let compressed_filepath = format!("{}.huff", filepath); // XXX: Is there a more beatiful way of doing this?

        std::fs::write(compressed_filepath.clone(), serial)
            .expect("Writing correctly to the compressed file");

        Ok(compressed_filepath)
    }
}

impl HuffmanFreqTree {
    pub fn weight(self: &Self) -> usize {
        self.tree.weight(self.frequencies)
    }

    fn fill_codewords(self: &Self, codewords: &mut Codewords) {
        self.tree.fill_codewords_with_acc(codewords, Vec::new());
    }
}

impl Ord for HuffmanFreqTree {
    // Custom comparison function for min-heap
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match self.tree.weight(self.frequencies).cmp(&other.tree.weight(other.frequencies)) {
            Equal => if self.tree == other.tree { Equal } else { Greater },
            x => x.reverse()    // to make min-heap
        }
    }
}

impl Huffman {

    fn text_to_flattened_paths(self: &Self, codewords: &Codewords) -> Vec<Sense> {
        let mut paths = Vec::new();
        for c in self.text.chars() {
            let mut path = codewords.get(&c).unwrap().clone();
            paths.append(&mut path)
        }
        return paths
    }

    fn paths_to_encoded_chars(self: &Self, paths: &mut Vec<Sense>) -> Vec<u8> {
        let mut encoded_chars = Vec::with_capacity(paths.len() / 8);

        let padding_count = 8 - paths.len() % 8;
        for _ in 0..padding_count {
            paths.push(Sense::Left);
        }

        // Adding bit by bit each sense
        let mut path_iter = paths.into_iter();
        while let Some(sense) = path_iter.next() {
            let mut n: u8 = 0;
            for d in (0..=7).rev() {
                let b = match sense {
                    Sense::Left  => 0,
                    Sense::Right => 1
                };
                n |= b >> d;    // Add bit representing the sense to take
            }
            encoded_chars.push(n);
        }

        return encoded_chars
    }

    pub fn compress(self: &Self) -> SerialisedHuffmanTree {
        let mut codewords: Codewords = HashMap::new();
        self.freq_tree.fill_codewords(&mut codewords);

        let mut paths = self.text_to_flattened_paths(&codewords);
        let encoded_chars = self.paths_to_encoded_chars(&mut paths);

        return SerialisedHuffmanTree {
            tree: self.freq_tree.tree.clone(),
            encoded_chars
        };
    }

    pub fn from_file(filepath: &String) -> Result<Self, String> {
        let text = std::fs::read_to_string(filepath).unwrap();

        if text.len() == 0 {
            return Err("No content to compress".to_string())
        }

        let mut frequencies = [0; 256];
        for c in text.chars() {
            frequencies[c as usize] += 1;
        }

        let mut leaves: BinaryHeap<HuffmanTree> = BinaryHeap::new();

        for (i, freq) in frequencies.iter().enumerate() {
            if *freq > 0 {
                let c = char::from_u32(i as u32).unwrap();
                leaves.push(HuffmanTree::Leaf(c))
            }
        }

        while leaves.len() > 1 {
            let a = leaves.pop().unwrap();
            let b = leaves.pop().unwrap();
            let node = HuffmanTree::Node((Box::new(a), Box::new(b)));
            leaves.push(node);
        }

        return Ok(Huffman {
            freq_tree: HuffmanFreqTree {
                frequencies,
                tree: leaves.pop().unwrap()
            },
            text
        })

    }
}
