use std::borrow::Cow;
use libmdbx::{Database, DatabaseOptions, WriteMap, WriteFlags, TableFlags};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;

#[derive(Clone)]
pub struct InnerDatabase {
    db: Arc<Mutex<Database<WriteMap>>>,
}

pub trait SafeDatabase {

    fn new<P: AsRef<Path>>(path: P) -> Result<Self, libmdbx::Error> where Self: Sized;

    fn clone(&self) -> Self where Self: Sized;



    // 트레이트 메서드에 pub 키워드 제거 (트레이트 자체가 pub이므로 메서드도 pub)
    fn write(&self, key: &str, value: &str, table: &str) -> Result<(), libmdbx::Error>;

    fn read(&self, key: &str, table: &str) -> Result<Option<Vec<u8>>, libmdbx::Error>;

    fn read_all(&self, table: &str) -> Result<HashMap<Vec<u8>, Vec<u8>>, libmdbx::Error>;

    fn batch_write<K, V>(&self, items: &[(K, V)], table: &str) -> Result<(), libmdbx::Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;
}


impl SafeDatabase for InnerDatabase{

    fn new<P: AsRef<Path>>(path: P) -> Result<Self, libmdbx::Error> {
        let mut options = DatabaseOptions::default();
        options.max_tables = Some(100);
        let db = Database::<WriteMap>::open_with_options(path, options)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
        }
    }


    fn write(&self, key: &str, value: &str, table: &str) -> Result<(), libmdbx::Error> {
        let db = self.db.lock().expect("Failed to lock database mutex");
        let transaction = db.begin_rw_txn()?;
        let table = transaction.create_table(Some(table), TableFlags::default())?;

        transaction.put(&table, key, value, WriteFlags::default())?;
        transaction.commit()?;
        Ok(())
    }


    fn read(&self, key: &str, table: &str) -> Result<Option<Vec<u8>>, libmdbx::Error> {
        let db = self.db.lock().expect("Failed to lock database mutex");
        let transaction = db.begin_ro_txn()?;

        if let Ok(table) = transaction.open_table(Some(table)) {
            let result = transaction.get(&table, key.as_bytes())?;
            return Ok(result);
        }

        Ok(None)
    }

    fn read_all(&self, table: &str) -> Result<HashMap<Vec<u8>, Vec<u8>>, libmdbx::Error> {
        let mut map = HashMap::new();
        let db = self.db.lock().expect("Failed to lock database mutex");
        let transaction = db.begin_ro_txn()?;

        if let Ok(table) = transaction.open_table(Some(table)) {
            let cursor = transaction.cursor(&table)?;

            for item in cursor {
                let (key, value) = item?;
                let key_owned = key.to_vec();
                let value_owned = value.to_vec();
                map.insert(key_owned, value_owned);
            }
        }

        Ok(map)
    }


    fn batch_write<K, V>(&self, items: &[(K, V)], table: &str) -> Result<(), libmdbx::Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let db = self.db.lock().expect("Failed to lock database mutex");
        let transaction = db.begin_rw_txn()?;
        let table = transaction.create_table(Some(table), TableFlags::default())?;

        for (key, value) in items {
            transaction.put(&table, key, value, WriteFlags::default())?;
        }

        transaction.commit()?;
        Ok(())
    }
}




//pub fn read_db<'a, 'b>(path:&'a str, table:&'a str) -> Result<  HashMap::<Cow<'b, [u8]> , Cow<'b, [u8]>>, libmdbx::Error>{
//    let mut map = HashMap::<Cow<[u8]> , Cow<[u8]>>::new();
//    let db = open_db(path)?;
//    let transaction = db.begin_ro_txn()?;
//    let table = transaction.open_table(Some(table))?;
//
//    let cursor = transaction.cursor(&table)?;
//
//    for item in cursor {
//        let (key_ , value_) =  item?;
//        map.insert(key_, value_);
//    }
//
//    transaction.commit()?;
//
//
//    Ok(map)
//}  ---> WARNING! : libmdbx using unsafe, so , If we set the lifetime like above,  there will be evoked dangling reference problem.


