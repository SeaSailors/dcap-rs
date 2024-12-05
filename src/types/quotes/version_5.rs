use super::{body::*, CertData, QuoteHeader};
use crate::constants::{
    ENCLAVE_REPORT_LEN, SGX_QUOTE_BODY_TYPE, SGX_TEE_TYPE, TD10_QUOTE_BODY_TYPE, TD10_REPORT_LEN,
    TD15_QUOTE_BODY_TYPE, TD15_REPORT_LEN, TDX_TEE_TYPE,
};

#[derive(Clone, Debug)]
pub struct QuoteV5 {
                                            // Header of Quote data structure.
    pub header: QuoteHeader,                // [48 bytes]
                                            // This field is transparent (the user knows its internal structure).
                                            // Rest of the Quote data structure can be treated as opaque (hidden from the user).
    pub quote_body_type: u16,               // [2 bytes]
    pub quote_body_size: u32,               // [4 bytes]
    pub quote_body: QuoteBody,              // May either contain a SGX Enclave Report (384 bytes) or TD10 Report (584 bytes)
    pub signature_len: u32,                 // [4 bytes]
                                            // Size of the Quote Signature Data structure in bytes.
    pub signature: QuoteSignatureDataV4,    // [variable bytes]
}

impl QuoteV5 {
    pub fn from_bytes(raw_bytes: &[u8]) -> Self {
        let header = QuoteHeader::from_bytes(&raw_bytes[0..48]);
        let quote_body_type = u16::from_le_bytes([raw_bytes[48], raw_bytes[49]]);
        let quote_body_size = u32::from_le_bytes([
            raw_bytes[50],
            raw_bytes[50 + 1],
            raw_bytes[50 + 2],
            raw_bytes[50 + 3],
        ]);
        let quote_body;
        let mut offset: usize = 54;
        match (header.tee_type, quote_body_type) {
            (SGX_TEE_TYPE, SGX_QUOTE_BODY_TYPE) => {
                offset += ENCLAVE_REPORT_LEN;
                quote_body =
                    QuoteBody::SGXQuoteBody(EnclaveReport::from_bytes(&raw_bytes[54..offset]));
            }
            (TDX_TEE_TYPE, TD10_QUOTE_BODY_TYPE) => {
                offset += TD10_REPORT_LEN;
                quote_body =
                    QuoteBody::TD10QuoteBody(TD10ReportBody::from_bytes(&raw_bytes[54..offset]));
            }
            (TDX_TEE_TYPE, TD15_QUOTE_BODY_TYPE) => {
                offset += TD15_REPORT_LEN;
                quote_body =
                    QuoteBody::TD15QuoteBody(TD15ReportBody::from_bytes(&raw_bytes[54..offset]));
            }
            _ => {
                panic!("Unknown TEE type")
            }
        }
        let signature_len = u32::from_le_bytes([
            raw_bytes[offset],
            raw_bytes[offset + 1],
            raw_bytes[offset + 2],
            raw_bytes[offset + 3],
        ]);
        offset += 4;
        let signature_slice = &raw_bytes[offset..offset + signature_len as usize];
        let signature = QuoteSignatureDataV4::from_bytes(signature_slice);

        QuoteV5 {
            header,
            quote_body_type,
            quote_body_size,
            quote_body,
            signature_len,
            signature,
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuoteSignatureDataV4 {
    pub quote_signature: [u8; 64],          // [64 bytes]
                                            // ECDSA signature, the r component followed by the s component, 2 x 32 bytes.
                                            // Public part of the Attestation Key generated by the Quoting Enclave.
    pub ecdsa_attestation_key: [u8; 64],    // [64 bytes]
                                            // EC KT-I Public Key, the x-coordinate followed by the y-coordinate (on the RFC 6090 P-256 curve), 2 x 32 bytes.
                                            // Public part of the Attestation Key generated by the Quoting Enclave.
    pub qe_cert_data: CertData,             // [variable bytes]
                                            // QE Cert Data
}

impl QuoteSignatureDataV4 {
    pub fn from_bytes(raw_bytes: &[u8]) -> Self {
        let mut quote_signature = [0; 64];
        quote_signature.copy_from_slice(&raw_bytes[0..64]);
        let mut ecdsa_attestation_key = [0; 64];
        ecdsa_attestation_key.copy_from_slice(&raw_bytes[64..128]);
        let qe_cert_data = CertData::from_bytes(&raw_bytes[128..]);

        QuoteSignatureDataV4 {
            quote_signature,
            ecdsa_attestation_key,
            qe_cert_data,
        }
    }
}
