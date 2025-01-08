use super::*;
use nom::{
    bytes::complete::take,
    combinator::{flat_map, map, map_opt, map_res, verify},
    error::{Error, ErrorKind, ParseError},
    multi::{count, length_count, length_data},
    number::complete::{be_i64, be_u32, be_u64},
    sequence::{pair, tuple},
    IResult, Parser,
};
use num_traits::FromPrimitive;

pub fn utf8str_cs<'a, const LIMIT: u32>(input: &'a [u8]) -> IResult<&'a [u8], Utf8StrCs<'a>> {
    flat_map(verify(be_u32, |&length| length as u32 <= LIMIT), |length| {
        map(map_res(take(length as u32), std::str::from_utf8), Utf8StrCs)
    })(input)
}

pub fn utf8str_cis<'a, const LIMIT: u32>(input: &'a [u8]) -> IResult<&'a [u8], Utf8StrCis<'a>> {
    flat_map(verify(be_u32, |&length| length as u32 <= LIMIT), |length| {
        map(
            map_res(take(length as u32), std::str::from_utf8),
            Utf8StrCis,
        )
    })(input)
}

pub fn opaque<'a>(length: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Opaque<'a>> {
    map(
        pair(take(length as usize), take((4 - (length as usize % 4)) % 4)),
        |(data, _)| Opaque(data),
    )
}

pub fn variable_length_opaque<'a, const LIMIT: u32>(
    input: &'a [u8]
) -> IResult<&'a [u8], Opaque<'a>> {
    flat_map(
        verify(be_u32, |&length| length as usize <= LIMIT as usize),
        |length| opaque(length as u32),
    )(input)
}

pub fn variable_length_array<'a, O, E: ParseError<&'a [u8]>, F, const LIMIT: u32>(
    parser: F
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<O>, E>
where
    F: Parser<&'a [u8], O, E> + Clone,
{
    move |input: &'a [u8]| {
        let (input, length) = verify(be_u32, |&length| length as usize <= LIMIT as usize)(input)?;
        count(parser.clone(), length as usize)(input)
    }
}

pub fn bitmap4(input: &[u8]) -> IResult<&[u8], Bitmap4> {
    map(
        variable_length_array::<_, _, _, { u32::MAX }>(be_u32),
        Bitmap4,
    )(input)
}

pub fn nfs_opnum4(input: &[u8]) -> IResult<&[u8], NfsOpnum4> {
    map_opt(be_u32, NfsOpnum4::from_u32)(input)
}

pub fn nfsstat4(input: &[u8]) -> IResult<&[u8], NfsStat4> {
    map_opt(be_u32, NfsStat4::from_u32)(input)
}

pub fn state_protect_ops4(input: &[u8]) -> IResult<&[u8], StateProtectOps4> {
    map(
        tuple((bitmap4, bitmap4)),
        |(spo_must_enforce, spo_must_allow)| StateProtectOps4 {
            spo_must_enforce,
            spo_must_allow,
        },
    )(input)
}

pub fn verifier4(input: &[u8]) -> IResult<&[u8], Verifier4> {
    map(opaque(NFS4_VERIFIER_SIZE), Verifier4)(input)
}

pub fn client_owner4(input: &[u8]) -> IResult<&[u8], ClientOwner4> {
    map(
        tuple((verifier4, variable_length_opaque::<NFS4_OPAQUE_LIMIT>)),
        |(co_verifier, co_ownerid)| ClientOwner4 {
            co_verifier,
            co_ownerid,
        },
    )(input)
}

pub fn nfstime4(input: &[u8]) -> IResult<&[u8], NfsTime4> {
    map(tuple((be_i64, be_u64)), |(seconds, nseconds)| NfsTime4 {
        seconds,
        nseconds,
    })(input)
}

pub fn ssv_sp_parms4(input: &[u8]) -> IResult<&[u8], SsvSpParms4> {
    map(
        tuple((
            state_protect_ops4,
            variable_length_array::<_, _, _, { u32::MAX }>(sec_oid4),
            variable_length_array::<_, _, _, { u32::MAX }>(sec_oid4),
            be_u32,
            be_u32,
        )),
        |(ssp_ops, ssp_hash_algs, ssp_encr_algs, ssp_window, ssp_num_gss_handles)| SsvSpParms4 {
            ssp_ops,
            ssp_hash_algs,
            ssp_encr_algs,
            ssp_window,
            ssp_num_gss_handles,
        },
    )(input)
}

pub fn sec_oid4(input: &[u8]) -> IResult<&[u8], SecOid4> {
    map(variable_length_opaque::<{ u32::MAX }>, SecOid4)(input)
}

pub fn nfs_impl_id4(input: &[u8]) -> IResult<&[u8], NfsImplId4> {
    map(
        tuple((
            utf8str_cis::<{ u32::MAX }>,
            utf8str_cs::<{ u32::MAX }>,
            nfstime4,
        )),
        |(nii_domain, nii_name, nii_date)| NfsImplId4 {
            nii_domain,
            nii_name,
            nii_date,
        },
    )(input)
}

pub fn compound4_args(input: &[u8]) -> IResult<&[u8], Compound4Args> {
    map(
        tuple((
            utf8str_cs::<{ u32::MAX }>,
            be_u32,
            variable_length_array::<_, _, _, { u32::MAX }>(nfs_argop4),
        )),
        |(tag, minorversion, argarray)| Compound4Args {
            tag,
            minorversion,
            argarray,
        },
    )(input)
}

pub fn exchange_id4_args(input: &[u8]) -> IResult<&[u8], ExchangeId4Args> {
    let (input, eia_clientowner) = client_owner4(input)?;
    let (input, eia_flags) = be_u32(input)?;
    let spa_how = StateProtectHow4::SP4_NONE;
    let (input, eia_state_protect) = state_protect4_a(spa_how)(input)?;
    let (input, eia_client_impl_id) = variable_length_array::<_, _, _, 1>(nfs_impl_id4)(input)?;
    let eia_client_impl_id = eia_client_impl_id.into_iter().next();
    Ok((
        input,
        ExchangeId4Args {
            eia_clientowner,
            eia_flags,
            eia_state_protect,
            eia_client_impl_id,
        },
    ))
}

pub fn nfs_argop4(input: &[u8]) -> IResult<&[u8], NfsArgOp4> {
    let (input, opnum) = nfs_opnum4(input)?;
    use NfsOpnum4::*;
    match opnum {
        EXCHANGE_ID4 => map(exchange_id4_args, NfsArgOp4::OP_EXCHANGE_ID)(input),
        _ => todo!(),
    }
}

pub fn state_protect4_a<'a>(
    spa_how: StateProtectHow4
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], StateProtect4A> {
    move |input: &'a [u8]| {
        use StateProtectHow4::*;
        Ok(match spa_how {
            SP4_NONE => (input, StateProtect4A::SP4_NONE),
            SP4_MACH_CRED => {
                let (input, spa_mach_ops) = state_protect_ops4(input)?;
                (input, StateProtect4A::SP4_MACH_CRED { spa_mach_ops })
            }
            SP4_SSV => {
                let (input, spa_ssv_parms) = ssv_sp_parms4(input)?;
                (input, StateProtect4A::SP4_SSV { spa_ssv_parms })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_length_bitmap4() {
        let input = &[0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03];
        let (input, result) = bitmap4(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, Bitmap4(vec![0x00010203]));
    }

    #[test]
    fn test_nfs_opnum4() {
        let input = &[0x00, 0x00, 0x00, 0x03];
        let (input, result) = nfs_opnum4(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, NfsOpnum4::OP_ACCESS);
    }

    #[test]
    fn test_nfsstat4() {
        let input = &[0x00, 0x00, 0x27, 0x39];
        let (input, result) = nfsstat4(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, NfsStat4::NFS4ERR_BADNAME);
    }
}
