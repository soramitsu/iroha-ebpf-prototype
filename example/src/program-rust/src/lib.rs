sdk::entrypoint!(contract);

pub fn contract(name: &str) {
    if sdk::balance(name) < 10 {
        sdk::mint(name, 1);
    } else {
        sdk::burn(name, 1);
    }
}
