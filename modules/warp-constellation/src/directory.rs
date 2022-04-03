use crate::item::{Item, Metadata};
use warp_common::chrono::{DateTime, Utc};
use warp_common::derive_more::Display;
use warp_common::serde::{Deserialize, Serialize};
use warp_common::uuid::Uuid;
use warp_common::{error::Error, Result};
/// `DirectoryType` handles the supported types for the directory.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display)]
#[serde(crate = "warp_common::serde", rename_all = "lowercase")]
pub enum DirectoryType {
    #[display(fmt = "default")]
    Default,
}

impl Default for DirectoryType {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Display)]
#[serde(crate = "warp_common::serde", rename_all = "lowercase")]
pub enum DirectoryHookType {
    #[display(fmt = "create")]
    Create,
    #[display(fmt = "delete")]
    Delete,
    #[display(fmt = "rename")]
    Rename,
    #[display(fmt = "move")]
    Move,
}

/// `Directory` handles folders and its contents.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "warp_common::serde")]
pub struct Directory {
    /// ID of the `Directory`
    pub id: Uuid,

    /// Name of the `Directory`
    pub name: String,

    /// Description of the `Directory`
    /// TODO: Make this optional
    pub description: String,

    /// Timestamp of the creation of the directory
    #[serde(with = "warp_common::chrono::serde::ts_seconds")]
    pub creation: DateTime<Utc>,

    /// Timestamp of the `Directory` when it is modified
    #[serde(with = "warp_common::chrono::serde::ts_seconds")]
    pub modified: DateTime<Utc>,

    /// Type of `Directory`
    pub directory_type: DirectoryType,

    /// List of `Item`, which would represents either `File` or `Directory`
    pub items: Vec<Item>,
}

impl Default for Directory {
    fn default() -> Self {
        let timestamp = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: String::from("un-named directory"),
            description: String::new(),
            creation: timestamp,
            modified: timestamp,
            directory_type: DirectoryType::Default,
            items: Vec::new(),
        }
    }
}

impl Directory {
    /// Create a `Directory` instance
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///
    ///     let dir = Directory::new("Test Directory");
    ///     assert_eq!(dir.name, String::from("Test Directory"));
    /// ```
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let mut directory = Directory::default();
        let name = name.as_ref().trim();
        if !name.is_empty() {
            directory.name = name.to_string();
        }
        directory
    }

    /// Recurseively create item `Directory` with each instance is a item to the previous
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory, item::Item};
    ///
    ///     let root = Directory::new_recursive("/root/test/test2").unwrap();
    ///     assert_eq!(root.has_child("test"), true);
    ///     let test = root.get_child("test").and_then(Item::get_directory).unwrap();
    ///     assert_eq!(test.has_child("test2"), true);
    /// ```
    pub fn new_recursive<S: AsRef<str>>(path: S) -> Result<Self> {
        let path = path.as_ref();

        // Check to determine if the string is initially empty
        if path.is_empty() {
            return Err(Error::InvalidPath);
        }

        let mut path = path
            .split('/')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<_>>();

        // checl to determine if the array is empty
        if path.is_empty() {
            return Err(Error::InvalidPath);
        }

        let name = path.remove(0);
        let mut directory = Self::new(name);
        if !path.is_empty() {
            let sub = Self::new_recursive(path.join("/"))?;
            directory.add_item(sub)?;
        }
        Ok(directory)
    }

    /// List all the `Item` within the `Directory`
    pub fn get_items(&self) -> &Vec<Item> {
        &self.items
    }

    /// List all mutable `Item` within the `Directory`
    pub fn get_items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.items
    }

    pub fn child_list(&self) -> &Vec<Item> {
        self.get_items()
    }

    pub fn child_list_mut(&mut self) -> &mut Vec<Item> {
        self.get_items_mut()
    }

    /// Checks to see if the `Directory` has a `Item`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory, DirectoryType}, item::Item};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///
    ///     assert_eq!(root.has_item("Sub Directory"), true);
    /// ```
    pub fn has_item<S: AsRef<str>>(&self, item_name: S) -> bool {
        self.get_items()
            .iter()
            .filter(|item| item.name() == item_name.as_ref())
            .count()
            == 1
    }

    pub fn has_child<S: AsRef<str>>(&self, child_name: S) -> bool {
        self.has_item(child_name)
    }

    /// Add an item to the `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory, DirectoryType}, item::Item};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_item(sub).unwrap();
    /// ```
    pub fn add_item<I: Into<Item>>(&mut self, item: I) -> Result<()> {
        //TODO: Implement check to make sure that the Directory isnt being added to itself
        let item = &item.into();
        if self.has_child(item.name()) {
            return Err(Error::DuplicateName);
        }

        if let Item::Directory(directory) = item {
            if directory == self {
                return Err(Error::DirParadox);
            }
        }
        self.items.push(item.clone());
        self.modified = Utc::now();
        Ok(())
    }

    pub fn add_child<I: Into<Item>>(&mut self, child: I) -> Result<()> {
        self.add_item(child)
    }

    /// Used to get the position of a child within a `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///     use warp_common::error::Error;
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub1 = Directory::new("Sub1 Directory");
    ///     let sub2 = Directory::new("Sub2 Directory");
    ///     root.add_child(sub1).unwrap();
    ///     root.add_child(sub2).unwrap();
    ///     assert_eq!(root.get_item_index("Sub1 Directory").unwrap(), 0);
    ///     assert_eq!(root.get_item_index("Sub2 Directory").unwrap(), 1);
    ///     assert_eq!(root.get_item_index("Sub3 Directory").is_err(), true);
    ///
    /// ```
    pub fn get_item_index<S: AsRef<str>>(&self, item_name: S) -> Result<usize> {
        self.items
            .iter()
            .position(|item| item.name() == item_name.as_ref())
            .ok_or(Error::ArrayPositionNotFound)
    }

    pub fn get_child_index<S: AsRef<str>>(&self, child_name: S) -> Result<usize> {
        self.get_item_index(child_name)
    }

    /// Used to get the child within a `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_item(sub).unwrap();
    ///     assert_eq!(root.has_item("Sub Directory"), true);
    ///     let item = root.get_item("Sub Directory").unwrap();
    ///     assert_eq!(item.name(), "Sub Directory");
    /// ```
    pub fn get_item<S: AsRef<str>>(&self, item_name: S) -> Result<&Item> {
        let item_name = item_name.as_ref();
        if !self.has_child(item_name) {
            return Err(Error::ItemInvalid);
        }
        let index = self.get_child_index(item_name)?;
        self.items.get(index).ok_or(Error::ItemInvalid)
    }

    pub fn get_child<S: AsRef<str>>(&self, child_name: S) -> Result<&Item> {
        self.get_item(child_name)
    }

    /// Used to get the mutable child within a `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory},file::File};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///     assert_eq!(root.has_child("Sub Directory"), true);
    ///     let child = root.get_child("Sub Directory").unwrap();
    ///     assert_eq!(child.name(), "Sub Directory");
    ///     let mut file = File::new("testFile.jpg");
    ///     root.get_child_mut("Sub Directory").unwrap()
    ///         .get_directory_mut().unwrap()
    ///         .add_child(file).unwrap();
    ///
    ///     let dir = root.get_child("Sub Directory").unwrap().get_directory().unwrap();
    ///
    ///     assert_eq!(dir.has_child("testFile.jpg"), true);
    ///     assert_eq!(dir.has_child("testFile.png"), false);
    ///
    /// ```
    pub fn get_item_mut<S: AsRef<str>>(&mut self, item_name: S) -> Result<&mut Item> {
        let item_name = item_name.as_ref();
        if !self.has_child(item_name) {
            return Err(Error::ItemInvalid);
        }
        let index = self.get_child_index(item_name)?;
        self.items.get_mut(index).ok_or(Error::ItemInvalid)
    }

    pub fn get_child_mut<S: AsRef<str>>(&mut self, child_name: S) -> Result<&mut Item> {
        self.get_item_mut(child_name)
    }

    /// Used to rename a child within a `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///     assert_eq!(root.has_child("Sub Directory"), true);
    ///
    ///     root.rename_child("Sub Directory", "Test Directory").unwrap();
    ///
    ///     assert_eq!(root.has_child("Test Directory"), true);
    ///
    /// ```
    pub fn rename_item<S: AsRef<str>>(&mut self, current_name: S, new_name: S) -> Result<()> {
        self.get_child_mut_by_path(current_name)?.rename(new_name)
    }

    pub fn rename_child<S: AsRef<str>>(&mut self, current_name: S, new_name: S) -> Result<()> {
        self.rename_item(current_name, new_name)
    }

    /// Used to remove the child within a `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///
    ///     assert_eq!(root.has_item("Sub Directory"), true);
    ///     let _ = root.remove_item("Sub Directory").unwrap();
    ///     assert_eq!(root.has_item("Sub Directory"), false);
    /// ```
    pub fn remove_item<S: AsRef<str>>(&mut self, item_name: S) -> Result<Item> {
        let item_name = item_name.as_ref();
        if !self.has_child(item_name) {
            return Err(Error::ItemInvalid);
        }
        let index = self.get_child_index(item_name)?;
        let item = self.items.remove(index);
        self.modified = Utc::now();
        Ok(item)
    }

    pub fn remove_child<S: AsRef<str>>(&mut self, child_name: S) -> Result<Item> {
        self.remove_item(child_name)
    }

    /// Used to remove the child within a `Directory` path
    ///
    /// TODO: Implement within `Directory::remove_child` in a single path
    ///
    /// # Examples
    ///
    /// ```
    ///         use warp_constellation::directory::{Directory};
    ///
    ///         let mut root = Directory::new("Test Directory");
    ///         let sub0 = Directory::new("Sub Directory 1");
    ///         let sub1 = Directory::new("Sub Directory 2");
    ///         let sub2 = Directory::new("Sub Directory 3");
    ///         root.add_item(sub0).unwrap();
    ///         root.add_item(sub1).unwrap();
    ///         root.add_item(sub2).unwrap();
    ///
    ///         assert_eq!(root.has_item("Sub Directory 1"), true);
    ///         assert_eq!(root.has_item("Sub Directory 2"), true);
    ///         assert_eq!(root.has_item("Sub Directory 3"), true);
    ///
    ///         root.move_item_to("Sub Directory 2", "Sub Directory 1").unwrap();
    ///         root.move_item_to("Sub Directory 3", "Sub Directory 1/Sub Directory 2").unwrap();
    ///
    ///         assert_ne!(root.has_item("Sub Directory 2"), true);
    ///         assert_ne!(root.has_item("Sub Directory 3"), true);
    ///
    ///         root.remove_item_from_path("/Sub Directory 1/Sub Directory 2", "Sub Directory 3").unwrap();
    ///
    ///         assert_eq!(root.get_item_by_path("Sub Directory 1/Sub Directory 2/Sub Directory 3").is_err(), true);
    /// ```
    pub fn remove_item_from_path<S: AsRef<str>>(&mut self, directory: S, item: S) -> Result<Item> {
        self.get_child_mut_by_path(directory)?
            .get_directory_mut()?
            .remove_child(item)
    }

    pub fn remove_child_from_path<S: AsRef<str>>(
        &mut self,
        directory: S,
        child: S,
    ) -> Result<Item> {
        self.remove_item_from_path(directory, child)
    }

    /// Used to move the child to another `Directory`
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub0 = Directory::new("Sub Directory 1");
    ///     let sub1 = Directory::new("Sub Directory 2");
    ///     root.add_child(sub0).unwrap();
    ///     root.add_child(sub1).unwrap();
    ///
    ///     assert_eq!(root.has_child("Sub Directory 1"), true);
    ///     assert_eq!(root.has_child("Sub Directory 2"), true);
    ///
    ///     root.move_item_to("Sub Directory 2", "Sub Directory 1").unwrap();
    ///
    ///     assert_ne!(root.has_child("Sub Directory 2"), true);
    /// ```
    pub fn move_item_to<S: AsRef<str>>(&mut self, child: S, dst: S) -> Result<()> {
        let (child, dst) = (child.as_ref().trim(), dst.as_ref().trim());

        if self.get_item_by_path(dst)?.is_file() {
            return Err(Error::ItemNotDirectory);
        }

        if self
            .get_child_by_path(dst)?
            .get_directory()?
            .has_child(child)
        {
            return Err(Error::DuplicateName);
        }

        let item = self.remove_child(child)?;

        // If there is an error, place the item back into the current directory
        match self
            .get_child_mut_by_path(dst)
            .and_then(Item::get_directory_mut)
        {
            Ok(directory) => {
                if let Err(e) = directory.add_child(item.clone()) {
                    self.add_child(item)?;
                    return Err(e);
                }
            }
            Err(e) => {
                self.add_child(item)?;
                return Err(e);
            }
        }
        Ok(())
    }

    /// Used to find an item throughout the `Directory` and its children
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let mut sub0 = Directory::new("Sub Directory 1");
    ///     let sub1 = Directory::new("Sub Directory 2");
    ///     sub0.add_child(sub1).unwrap();
    ///     root.add_child(sub0).unwrap();
    ///
    ///     assert_eq!(root.has_child("Sub Directory 1"), true);
    ///
    ///     let item = root.find_item("Sub Directory 2").unwrap();
    ///     assert_eq!(item.name(), "Sub Directory 2");
    /// ```
    pub fn find_item<S: AsRef<str>>(&self, item_name: S) -> Result<&Item> {
        let item_name = item_name.as_ref();
        for item in self.items.iter() {
            if item.name().eq(item_name) {
                return Ok(item);
            }
            if let Item::Directory(directory) = item {
                match directory.find_item(item_name) {
                    Ok(item) => return Ok(item),
                    //Since we know the error will be `Error::ItemInvalid`, we can continue through the iteration until it ends
                    Err(_) => continue,
                }
            }
        }
        Err(Error::ItemInvalid)
    }

    /// Used to get a search for items listed and return a list of `Item` matching the terms
    ///
    /// # Examples
    ///
    /// TODO
    pub fn find_all_items<S: AsRef<str> + Clone>(&self, item_names: Vec<S>) -> Vec<&Item> {
        let mut list = Vec::new();
        for item in self.items.iter() {
            for name in item_names.iter().map(|name| name.as_ref()) {
                if item.name().contains(name) {
                    list.push(item);
                }
            }

            if let Item::Directory(directory) = item {
                list.extend(directory.find_all_items(item_names.clone()));
            }
        }
        list
    }

    /// Get an `Item` from a path
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///     let mut root = Directory::new("Test Directory");
    ///     let mut sub0 = Directory::new("Sub Directory 1");
    ///     let mut sub1 = Directory::new("Sub Directory 2");
    ///     let sub2 = Directory::new("Sub Directory 3");
    ///     sub1.add_child(sub2).unwrap();
    ///     sub0.add_child(sub1).unwrap();
    ///     root.add_child(sub0).unwrap();
    ///
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/Sub Directory 3").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/Sub Directory3/Another Dir").is_ok(), false);
    /// ```
    pub fn get_item_by_path<S: AsRef<str>>(&self, path: S) -> Result<&Item> {
        let mut path = path
            .as_ref()
            .split('/')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<_>>();
        if path.is_empty() {
            return Err(Error::InvalidPath);
        }
        let name = path.remove(0);
        let item = self.get_item(name)?;
        return if !path.is_empty() {
            if let Item::Directory(dir) = item {
                dir.get_item_by_path(path.join("/"))
            } else {
                Ok(item)
            }
        } else {
            Ok(item)
        };
    }

    pub fn get_child_by_path<S: AsRef<str>>(&self, path: S) -> Result<&Item> {
        self.get_item_by_path(path)
    }

    /// Get a mutable `Item` from a path
    ///
    /// # Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///     let mut root = Directory::new("Test Directory");
    ///     let mut sub0 = Directory::new("Sub Directory 1");
    ///     let mut sub1 = Directory::new("Sub Directory 2");
    ///     let sub2 = Directory::new("Sub Directory 3");
    ///     sub1.add_child(sub2).unwrap();
    ///     sub0.add_child(sub1).unwrap();
    ///     root.add_child(sub0).unwrap();
    ///     
    ///     root.get_child_mut_by_path("/Sub Directory 1/Sub Directory 2").unwrap()
    ///         .get_directory_mut().unwrap()
    ///         .add_child(Directory::new("Another Directory")).unwrap();    
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/Sub Directory 3").is_ok(), true);
    ///     assert_eq!(root.get_child_by_path("/Sub Directory 1/Sub Directory 2/Another Directory").is_ok(), true);
    /// ```
    pub fn get_item_mut_by_path<S: AsRef<str>>(&mut self, path: S) -> Result<&mut Item> {
        let mut path = path
            .as_ref()
            .split('/')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<_>>();
        if path.is_empty() {
            return Err(Error::InvalidPath);
        }
        let name = path.remove(0);
        let item = self.get_item_mut(name)?;
        return if !path.is_empty() {
            if let Item::Directory(dir) = item {
                dir.get_item_mut_by_path(path.join("/"))
            } else {
                Ok(item)
            }
        } else {
            Ok(item)
        };
    }

    pub fn get_child_mut_by_path<S: AsRef<str>>(&mut self, path: S) -> Result<&mut Item> {
        self.get_item_mut_by_path(path)
    }
}

impl Metadata for Directory {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn description(&self) -> String {
        self.description.to_owned()
    }

    fn size(&self) -> i64 {
        self.items.iter().map(Metadata::size).sum()
    }

    fn creation(&self) -> DateTime<Utc> {
        self.creation
    }

    fn modified(&self) -> DateTime<Utc> {
        self.modified
    }
}
