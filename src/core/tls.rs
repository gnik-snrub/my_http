use std::{fs::File, io::{BufReader}};

use rustls::{pki_types::{CertificateDer, PrivateKeyDer}};
use rustls_pemfile::{certs, private_key};

pub fn load_certs_and_key() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), std::io::Error>{
    dotenvy::dotenv().ok();
    let cert_path = std::env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH not set");
    let key_path = std::env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH not set");

    let mut cert_file = &mut BufReader::new(File::open(cert_path).unwrap());
    let mut key_file = &mut BufReader::new(File::open(key_path).unwrap());

    let cert: Vec<CertificateDer<'static>> = certs(&mut cert_file).collect::<Result<_,_>>()?;
    let key: PrivateKeyDer<'static> = private_key(&mut key_file)?
        .expect("No private key found in key.pem");

    Ok((cert, key))
}
