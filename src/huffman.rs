
use std::{collections::{BinaryHeap, HashMap}, io::{BufWriter, Write}};

use serde::{Serialize, Deserialize};
use postcard;

#[derive(Eq, PartialEq, PartialOrd,
         Clone, Debug,
         Serialize, Deserialize)]
enum HuffmanTree {
    Leaf(char),
    Node((Box<HuffmanTree>, Box<HuffmanTree>))
}

#[derive(Serialize, Deserialize)]
pub struct SerialisedHuffmanTree {
    tree: HuffmanTree,
    senses_count: usize,
    encoded_chars: Vec<u8>
}

type Frequencies = [usize; 256];

#[derive(Eq, PartialEq, Debug)]
pub struct HuffmanFreqTree {
    frequencies: Frequencies,
    tree: HuffmanTree
}

#[derive(Debug)]
pub struct Huffman {
    freq_tree: HuffmanFreqTree,
    text: String
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
enum Sense {
    Left,
    Right
}

type Path = Vec<Sense>;
type Codewords = HashMap<char, Path>;
type CodewordsRev = HashMap<Path, char>;

pub static COMPRESSED_FILE_EXTENSION: &'static str = "huff";

impl HuffmanTree {
    pub fn weight(self: &Self, frequencies: Frequencies) -> usize {
         match self {
            HuffmanTree::Leaf(c) => {
                let n = *c as usize;
                frequencies[n]
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
                let mut left_path = current_path.clone();
                let mut right_path = current_path.clone();
                left_path.push(Sense::Left);
                right_path.push(Sense::Right);
                s.fill_codewords_with_acc(codewords, left_path);
                t.fill_codewords_with_acc(codewords, right_path);
            }
        }
    }

    fn fill_codewords(self: &Self, codewords: &mut Codewords) {
        self.fill_codewords_with_acc(codewords, Vec::new());
    }

    fn fill_codewords_rev_with_acc(self: &Self, codewords_rev: &mut CodewordsRev, current_path: Path) {
        match self {
            HuffmanTree::Leaf(c) => {
                let _ = codewords_rev.insert(current_path, *c);
            },
            HuffmanTree::Node((s, t)) => {
                let mut left_path = current_path.clone();
                let mut right_path = current_path.clone();
                left_path.push(Sense::Left);
                right_path.push(Sense::Right);
                s.fill_codewords_rev_with_acc(codewords_rev, left_path);
                t.fill_codewords_rev_with_acc(codewords_rev, right_path);
            }
        }
    }

    fn fill_codewords_rev(self: &Self, codewords: &mut CodewordsRev) {
        self.fill_codewords_rev_with_acc(codewords, Vec::new());
    }
}

impl SerialisedHuffmanTree {
    pub fn serialise(self: &Self, filepath: String) -> Result<(String, usize), String> {
        let serial = postcard::to_allocvec(&self)
            .expect("A valid serialisation");
        let compressed_size = serial.len();
        let compressed_filepath = format!("{filepath}.{COMPRESSED_FILE_EXTENSION}"); // XXX: Is there a more beatiful way of doing this?

        std::fs::write(compressed_filepath.clone(), serial)
            .expect("Writing correctly to the compressed file");

        Ok((compressed_filepath, compressed_size))
    }

    pub fn deserialise(compressed_filepath: String) -> (Self, String) {
        let compressed = std::fs::read(&compressed_filepath).unwrap();

        let mut original_filepath_filebuf = std::path::PathBuf::from(&compressed_filepath);
        original_filepath_filebuf.set_extension(""); // Removes `.huff` extension
        let original_filepath = original_filepath_filebuf.into_os_string().into_string().unwrap();

        println!("{} --> {}", compressed_filepath, original_filepath);

        let deserial: SerialisedHuffmanTree = postcard::from_bytes(&compressed)
            .expect("Valid compressed file contents");

        return (deserial, original_filepath)
    }

    fn encoded_chars_to_senses(self: &Self) -> Vec<Sense> {
        let mut senses = Vec::new();
        let mut i = 0;

        for n in self.encoded_chars.clone().into_iter() {
            for offset in (0..=7).rev() {
                if i > self.senses_count {
                    break;
                }
                let sense = if ((n >> offset) & 1) == 1 { Sense::Right } else { Sense::Left };
                senses.push(sense);
                i += 1;
            }
        }

        return senses
    }
}

impl HuffmanFreqTree {
    pub fn weight(self: &Self) -> usize {
        self.tree.weight(self.frequencies)
    }
}

impl PartialOrd for HuffmanFreqTree {
    // Custom comparison function for min-heap
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.weight().cmp(&self.weight()))
    }
}

impl Ord for HuffmanFreqTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl Huffman {

    fn text_to_flattened_senses(self: &Self, codewords: &Codewords) -> Vec<Sense> {
        let mut paths = Vec::new();
        for c in self.text.chars() {
            let mut path = codewords.get(&c).unwrap().clone();
            paths.append(&mut path)
        }
        return paths
    }

    fn senses_to_encoded_chars(self: &Self, paths: &mut Vec<Sense>) -> (Vec<u8>, usize) {
        let mut encoded_chars = Vec::with_capacity(paths.len() / 8);

        let padding_count = 8 - paths.len() % 8;
        for _ in 0..padding_count {
            paths.push(Sense::Left);
        }

        // Adding each sense bit by bit
        let mut curr_path = 0;
        while curr_path < paths.len() {
            let mut n: u8 = 0;
            for offset in (0..=7).rev() {
                let bit = match paths[curr_path] {
                    Sense::Left  => 0,
                    Sense::Right => 1
                };
                n |= bit << offset; // Add bit representing the sense to take
                curr_path += 1;
            }
            encoded_chars.push(n);
        }

        assert!(paths.len() == curr_path);
        return (encoded_chars, curr_path - padding_count)
    }

    pub fn compress(self: &Self) -> SerialisedHuffmanTree {
        let mut codewords: Codewords = HashMap::new();
        self.freq_tree.tree.fill_codewords(&mut codewords);

        let mut senses = self.text_to_flattened_senses(&codewords);

        let (encoded_chars, senses_count) = self.senses_to_encoded_chars(&mut senses);

        return SerialisedHuffmanTree {
            tree: self.freq_tree.tree.clone(),
            senses_count,
            encoded_chars
        }
    }

    fn reconstruct_text(tree: &HuffmanTree, senses: Vec<Sense>) -> String {
        let mut codewords_rev: CodewordsRev = HashMap::new();
        tree.fill_codewords_rev(&mut codewords_rev);

        let mut reconstructed_text = String::new();

        let mut current_path: Vec<Sense> = Vec::new();
        for sense in senses {
            current_path.push(sense);
            if let Some(c) = codewords_rev.get(&current_path) {
                reconstructed_text.push(*c);
                current_path.clear();
            }
        }

        return reconstructed_text
    }

    pub fn decompress(deserial: SerialisedHuffmanTree) -> Option<String> {
        let senses = deserial.encoded_chars_to_senses();
        let reconstructed_text = Huffman::reconstruct_text(&deserial.tree, senses);

        Some(reconstructed_text)
    }

    pub fn from_file(filepath: &String) -> Result<(Self, usize), String> {
        let text = std::fs::read_to_string(filepath).unwrap();
        let text_len = text.len();

        if text_len == 0 {
            return Err("No content to compress".to_string())
        }

        let mut frequencies = [0; 256];
        for c in text.chars() {
            frequencies[c as usize] += 1;
        }

        let mut leaves: BinaryHeap<HuffmanFreqTree> = BinaryHeap::new();

        for (i, freq) in frequencies.iter().enumerate() {
            if *freq > 0 {
                let c = char::from_u32(i as u32).unwrap();
                leaves.push(HuffmanFreqTree {
                    frequencies,
                    tree: HuffmanTree::Leaf(c)
                })
            }
        }

        while leaves.len() > 1 {
            let a = leaves.pop().unwrap().tree;
            let b = leaves.pop().unwrap().tree;
            let node = HuffmanFreqTree {
                frequencies,
                tree: HuffmanTree::Node((Box::new(a), Box::new(b)))
            };
            leaves.push(node);
        }

        let huf = Huffman {
            freq_tree: leaves.pop().unwrap(),
            text
        };

        return Ok((huf, text_len))

    }
}
