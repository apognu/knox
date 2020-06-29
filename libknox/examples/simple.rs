use libknox::*;

fn main() {
  // Create a new vault with the given GPG identity
  let id = vec!["6A25FCF213C7779AD26DC50706CB643B42E7CD3E".to_string()];
  let mut vault = VaultContext::create("/tmp/knox-example", &id).expect("FAIL");

  // Create a new entry with three attributes
  let mut entry = Entry::new();
  entry.add_attribute("username", "bob");
  entry.add_confidential_attribute("password", "foobar");
  entry.add_confidential_attribute("apikey", "3OJL07P+W5zODH2J1Wv7rXh5i9UpR0mpvPW7ygIMih82J8P95krJZXyERqbi/XS");

  // Write the entry and the metadata pointing to it
  vault.write_entry("personal/website.com", &entry).expect("FAIL");

  // Open the previously created vault and read the written entry
  let vault = VaultContext::open("/tmp/knox-example").expect("FAIL");
  let entry = vault.read_entry("personal/website.com").expect("FAIL");

  // Loop over the attributes and print them
  for (key, attribute) in entry.get_attributes() {
    if attribute.confidential {
      println!("{} = {} (CONFIDENTIAL)", key, attribute.value);
    } else {
      println!("{} = {}", key, attribute.value);
    }
  }
}
