use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::Error;
use crate::item::{Item, ItemMeta};

/// `DirectoryType` handles the supported types for the directory.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum DirectoryType {
    Default
}

impl Default for DirectoryType {
    fn default() -> Self {
        Self::Default
    }
}

/// `Directory` handles folders and its contents.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Directory {
    #[serde(flatten)]
    pub metadata: ItemMeta,
    pub directory_type: DirectoryType,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub children: Vec<Item>,
}

impl Default for Directory {

    fn default() -> Self {
        Self {
            metadata: ItemMeta {
                id: Uuid::new_v4(),
                name: String::from("un-named directory"),
                description: String::new(),
                size: None
            },
            parent: None,
            directory_type: DirectoryType::Default,
            children: Vec::new()
        }
    }

}

impl Directory {
    /// Create a `Directory` instance
    ///
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///
    ///     let dir = Directory::new("Test Directory");
    ///     assert_eq!(dir.metadata.name, String::from("Test Directory"));
    /// ```
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let mut directory = Directory::default();
        let name = name.as_ref().trim();
        if name.len() >= 1 { directory.metadata.name = name.to_string(); }
        directory
    }

    /// List all the `Item` within the `Directory`
    pub fn child_list(&self) -> &Vec<Item> { &self.children }

    /// List all mutable `Item` within the `Directory`
    pub fn child_list_mut(&mut self) -> &mut Vec<Item> { &mut self.children }

    /// Checks to see if the `Directory` has a `Item`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory, DirectoryType}, item::Item};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///
    ///     assert_eq!(root.has_child("Sub Directory"), true);
    /// ```
    pub fn has_child<S: AsRef<str>>(&self, child_name: S) -> bool {
        self.child_list()
            .iter()
            .filter(|item| item.name() == child_name.as_ref())
            .collect::<Vec<_>>().len() == 1
    }

    /// Add an item to the `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory, DirectoryType}, item::Item};
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    /// ```
    pub fn add_child<I: Into<Item>>(&mut self, child: I) -> Result<(), Error> {
        //TODO: Implement check to make sure that the Directory isnt being added to itself
        let child = &child.into();
        if self.has_child(child.name()) { return Err(Error::DuplicateName) }

        if let Item::Directory(directory) = child {
            if directory == self { return Err(Error::DirParadox); }
        }
        self.children.push(child.clone());
        Ok(())
    }

    /// Used to get the position of a child within a `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///     use warp_constellation::error::Error;
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub1 = Directory::new("Sub1 Directory");
    ///     let sub2 = Directory::new("Sub2 Directory");
    ///     root.add_child(sub1).unwrap();
    ///     root.add_child(sub2).unwrap();
    ///     assert_eq!(root.get_child_index("Sub1 Directory").unwrap(), 0);
    ///     assert_eq!(root.get_child_index("Sub2 Directory").unwrap(), 1);
    ///     assert_eq!(root.get_child_index("Sub3 Directory").unwrap_err(), Error::ArrayPositionNotFound);
    ///
    /// ```
    pub fn get_child_index<S: AsRef<str>>(&self, child_name: S) -> Result<usize, Error> {
        self.children.iter().position(|item| item.name() == child_name.as_ref()).ok_or(Error::ArrayPositionNotFound)
    }

    /// Used to get the child within a `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///     use warp_constellation::error::Error;
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///     assert_eq!(root.has_child("Sub Directory"), true);
    ///     let child = root.get_child("Sub Directory").unwrap();
    ///     assert_eq!(child.name(), "Sub Directory");
    /// ```
    pub fn get_child<S: AsRef<str>>(&self, child_name: S) -> Result<&Item, Error> {
        let child_name = child_name.as_ref();
        if !self.has_child(child_name) { return Err(Error::ItemInvalid); }
        let index = self.get_child_index(child_name)?;
        self.children.get(index).ok_or(Error::ItemInvalid)
    }

    /// Used to get the mutable child within a `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory},file::File};
    ///     use warp_constellation::error::Error;
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
    pub fn get_child_mut<S: AsRef<str>>(&mut self, child_name: S) -> Result<&mut Item, Error> {
        let child_name = child_name.as_ref();
        if !self.has_child(child_name) { return Err(Error::ItemInvalid); }
        let index = self.get_child_index(child_name)?;
        self.children.get_mut(index).ok_or(Error::ItemInvalid)
    }

    /// Used to rename a child within a `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///     use warp_constellation::error::Error;
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
    pub fn rename_child<S: AsRef<str>>(&mut self, current_name: S, new_name: S) -> Result<(), Error> {
        self.get_child_mut_by_path(current_name)?
            .rename(new_name)
    }

    /// Used to remove the child within a `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::{Directory}};
    ///     use warp_constellation::error::Error;
    ///
    ///     let mut root = Directory::new("Test Directory");
    ///     let sub = Directory::new("Sub Directory");
    ///     root.add_child(sub).unwrap();
    ///
    ///     assert_eq!(root.has_child("Sub Directory"), true);
    ///     let _ = root.remove_child("Sub Directory").unwrap();
    ///     assert_eq!(root.has_child("Sub Directory"), false);
    /// ```
    pub fn remove_child<S: AsRef<str>>(&mut self, child_name: S) -> Result<Item, Error> {
        let child_name = child_name.as_ref();
        if !self.has_child(child_name) { return Err(Error::ItemInvalid); }
        let index = self.get_child_index(child_name)?;
        Ok(self.children.remove(index))
    }

    /// Used to remove the child within a `Directory` path
    ///
    /// TODO: Implement within `Directory::remove_child` in a single path
    ///
    /// #Examples
    ///
    /// ```
    ///         use warp_constellation::directory::{Directory};
    ///
    ///         let mut root = Directory::new("Test Directory");
    ///         let sub0 = Directory::new("Sub Directory 1");
    ///         let sub1 = Directory::new("Sub Directory 2");
    ///         let sub2 = Directory::new("Sub Directory 3");
    ///         root.add_child(sub0).unwrap();
    ///         root.add_child(sub1).unwrap();
    ///         root.add_child(sub2).unwrap();
    ///
    ///         assert_eq!(root.has_child("Sub Directory 1"), true);
    ///         assert_eq!(root.has_child("Sub Directory 2"), true);
    ///         assert_eq!(root.has_child("Sub Directory 3"), true);
    ///
    ///         root.move_item_to("Sub Directory 2", "Sub Directory 1").unwrap();
    ///         root.move_item_to("Sub Directory 3", "Sub Directory 1/Sub Directory 2").unwrap();
    ///
    ///         assert_ne!(root.has_child("Sub Directory 2"), true);
    ///         assert_ne!(root.has_child("Sub Directory 3"), true);
    ///
    ///         root.remove_child_from_path("/Sub Directory 1/Sub Directory 2", "Sub Directory 3").unwrap();
    ///
    ///         assert_eq!(root.get_child_by_path("Sub Directory 1/Sub Directory 2/Sub Directory 3").is_err(), true);
    /// ```
    pub fn remove_child_from_path<S: AsRef<str>>(&mut self, directory: S, child: S) -> Result<Item, Error> {
        self.get_child_mut_by_path(directory)?
            .get_directory_mut()?
            .remove_child(child)
    }
    /// Used to move the child to another `Directory`
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///     use warp_constellation::error::Error;
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
    pub fn move_item_to<S: AsRef<str>>(&mut self, child: S, dst: S) -> Result<(), Error> {
        let (child, dst) = (child.as_ref().trim(), dst.as_ref().trim());

        if self.get_child_by_path(dst)?.is_file() { return Err(Error::ItemNotDirectory); }

        if self.get_child_by_path(dst)?.get_directory()?.has_child(child) { return Err(Error::DuplicateName); }

        let item = self.remove_child(child)?;

        // TODO: Implement check and restore item back to previous directory if there's an error
        self.get_child_mut_by_path(dst)?
            .get_directory_mut()?
            .add_child(item)
    }

    /// Used to find an item throughout the `Directory` and its children
    ///
    /// #Examples
    ///
    /// ```
    ///     use warp_constellation::{directory::Directory};
    ///     use warp_constellation::error::Error;
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
    pub fn find_item<S: AsRef<str>>(&self, item_name: S) -> Result<&Item, Error> {
        let item_name = item_name.as_ref();
        for item in self.children.iter() {
            if item.name().eq(item_name) {
                return Ok(item);
            }
            if let Item::Directory(directory) = item {
                match directory.find_item(item_name) {
                    Ok(item) => return Ok(item),
                    //Since we know the error will be `Error::ItemInvalid`, we can continue through the iteration until it ends
                    Err(_) => continue
                }
            }
        }
        return Err(Error::ItemInvalid);
    }

    /// Used to get a search for items listed and return a list of `Item` matching the terms
    ///
    /// #Examples
    ///
    /// TODO
    pub fn find_all_items(&self, item_names: &Vec<&str>) -> Vec<&Item> {
        let mut list = Vec::new();
        for item in self.children.iter() {

            for name in item_names.iter() {
                if item.name().contains(name) { list.push(item); }
            }

            if let Item::Directory(directory) = item {
                list.extend(directory.find_all_items(item_names));
            }

        }
        list
    }

    /// Get an `Item` from a path
    ///
    /// # Examples
    ///
    /// TODO
    pub fn get_child_by_path<S: AsRef<str>>(&self, path: S) -> Result<&Item, Error> {
        let mut path = path.as_ref().split("/").filter(|&s| !s.is_empty()).collect::<Vec<_>>();
        loop {
            if path.is_empty() { return Err(Error::ItemInvalid); } //TODO: Use a different error term
            let name = path.remove(0);
            let item = self.get_child(name)?;
            return if !path.is_empty() {
                if let Item::Directory(dir) = item {
                    dir.get_child_by_path(path.join("/"))
                } else {
                    Ok(item)
                }
            } else {
                Ok(item)
            }
        }
    }

    /// Get a mutable `Item` from a path
    ///
    /// # Examples
    ///
    /// TODO
    pub fn get_child_mut_by_path<S: AsRef<str>>(&mut self, path: S) -> Result<&mut Item, Error> {
        let mut path = path.as_ref().split("/").filter(|&s| !s.is_empty()).collect::<Vec<_>>();
        loop {
            if path.is_empty() { return Err(Error::ItemInvalid); } //TODO: Use a different error term
            let name = path.remove(0);
            let item = self.get_child_mut(name)?;
            return if !path.is_empty() {
                if let Item::Directory(dir) = item {
                    dir.get_child_mut_by_path(path.join("/"))
                } else {
                    Ok(item)
                }
            } else {
                Ok(item)
            }
        }
    }
}