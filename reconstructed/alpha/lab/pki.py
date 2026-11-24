"""Issue short-lived localhost credentials for one temporary lab directory."""
from datetime import datetime, timedelta, timezone
from ipaddress import ip_address

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import ExtendedKeyUsageOID, NameOID


def issue(directory):
    now = datetime.now(timezone.utc)
    def certificate(label, *, ca=None, purpose=None):
        key = ec.generate_private_key(ec.SECP256R1())
        name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, label)])
        builder = (x509.CertificateBuilder().subject_name(name)
                   .issuer_name(ca[1].subject if ca else name)
                   .public_key(key.public_key()).serial_number(x509.random_serial_number())
                   .not_valid_before(now - timedelta(minutes=1)).not_valid_after(now + timedelta(days=1))
                   .add_extension(x509.BasicConstraints(ca=ca is None, path_length=None), critical=True))
        if purpose:
            builder = builder.add_extension(x509.ExtendedKeyUsage([purpose]), critical=False)
        builder = builder.add_extension(x509.SubjectKeyIdentifier.from_public_key(key.public_key()), False)
        builder = builder.add_extension(x509.AuthorityKeyIdentifier.from_issuer_public_key(
            (ca[0] if ca else key).public_key()), False)
        builder = builder.add_extension(x509.KeyUsage(
            digital_signature=True, content_commitment=False, key_encipherment=False,
            data_encipherment=False, key_agreement=False, key_cert_sign=ca is None,
            crl_sign=ca is None, encipher_only=None, decipher_only=None), True)
        if purpose == ExtendedKeyUsageOID.SERVER_AUTH:
            builder = builder.add_extension(x509.SubjectAlternativeName(
                [x509.IPAddress(ip_address("127.0.0.1"))]), critical=False)
        cert = builder.sign(ca[0] if ca else key, hashes.SHA256())
        (directory / (label + ".pem")).write_bytes(cert.public_bytes(serialization.Encoding.PEM))
        if ca:
            (directory / (label + ".key")).write_bytes(key.private_bytes(
                serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8,
                serialization.NoEncryption()))
        return key, cert

    authority = certificate("ca")
    certificate("server", ca=authority, purpose=ExtendedKeyUsageOID.SERVER_AUTH)
    certificate("client", ca=authority, purpose=ExtendedKeyUsageOID.CLIENT_AUTH)
