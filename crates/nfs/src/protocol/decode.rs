use super::*;
use nom::{
    bytes::complete::take,
    combinator::{fail, flat_map, map, map_opt, map_res, verify},
    error::ParseError,
    multi::{count, length_value},
    number::complete::{be_i64, be_u32, be_u64},
    sequence::{pair, tuple},
    IResult, Parser,
};
use num_traits::FromPrimitive;

fn utf8str_cs<'a, const LIMIT: u32>(input: &'a [u8]) -> IResult<&'a [u8], Utf8StrCs<'a>> {
    flat_map(verify(be_u32, |&length| length as u32 <= LIMIT), |length| {
        map(
            map_res(take(length as u32), std::str::from_utf8),
            Utf8StrCs::from,
        )
    })(input)
}

fn utf8str_cis<'a, const LIMIT: u32>(input: &'a [u8]) -> IResult<&'a [u8], Utf8StrCis<'a>> {
    flat_map(verify(be_u32, |&length| length as u32 <= LIMIT), |length| {
        map(
            map_res(take(length as u32), std::str::from_utf8),
            Utf8StrCis::from,
        )
    })(input)
}

fn opaque<'a>(length: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Opaque<'a>> {
    map(
        pair(take(length as usize), take((4 - (length as usize % 4)) % 4)),
        |(data, _)| Opaque::from(data),
    )
}

fn variable_length_opaque<'a, const LIMIT: u32>(input: &'a [u8]) -> IResult<&'a [u8], Opaque<'a>> {
    flat_map(
        verify(be_u32, |&length| length as usize <= LIMIT as usize),
        |length| opaque(length as u32),
    )(input)
}

fn variable_length_array<'a, O, E: ParseError<&'a [u8]>, F, const LIMIT: u32>(
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

fn bitmap(input: &[u8]) -> IResult<&[u8], Bitmap> {
    map(
        variable_length_array::<_, _, _, { u32::MAX }>(be_u32),
        Bitmap::from,
    )(input)
}

fn nfs_opnum(input: &[u8]) -> IResult<&[u8], NfsOpnum> {
    map_opt(be_u32, NfsOpnum::from_u32)(input)
}

fn nfsstat(input: &[u8]) -> IResult<&[u8], NfsStat> {
    map_opt(be_u32, NfsStat::from_u32)(input)
}

fn state_protect_ops(input: &[u8]) -> IResult<&[u8], StateProtectOps> {
    map(tuple((bitmap, bitmap)), |(must_enforce, must_allow)| {
        StateProtectOps {
            must_enforce,
            must_allow,
        }
    })(input)
}

fn verifier(input: &[u8]) -> IResult<&[u8], Verifier> {
    map(opaque(NFS4_VERIFIER_SIZE), Verifier)(input)
}

fn client_owner(input: &[u8]) -> IResult<&[u8], ClientOwner> {
    map(
        tuple((verifier, variable_length_opaque::<NFS4_OPAQUE_LIMIT>)),
        |(verifier, ownerid)| ClientOwner { verifier, ownerid },
    )(input)
}

fn nfstime(input: &[u8]) -> IResult<&[u8], NfsTime> {
    map(tuple((be_i64, be_u64)), |(seconds, nseconds)| NfsTime {
        seconds,
        nseconds,
    })(input)
}

fn ssv_sp_parms(input: &[u8]) -> IResult<&[u8], SsvSpParms> {
    map(
        tuple((
            state_protect_ops,
            variable_length_array::<_, _, _, { u32::MAX }>(sec_oid),
            variable_length_array::<_, _, _, { u32::MAX }>(sec_oid),
            be_u32,
            be_u32,
        )),
        |(ops, hash_algs, encr_algs, window, num_gss_handles)| SsvSpParms {
            ops,
            hash_algs,
            encr_algs,
            window,
            num_gss_handles,
        },
    )(input)
}

fn sec_oid(input: &[u8]) -> IResult<&[u8], SecOid> {
    map(variable_length_opaque::<{ u32::MAX }>, SecOid)(input)
}

fn nfs_impl_id(input: &[u8]) -> IResult<&[u8], NfsImplId> {
    map(
        tuple((
            utf8str_cis::<{ u32::MAX }>,
            utf8str_cs::<{ u32::MAX }>,
            nfstime,
        )),
        |(domain, name, date)| NfsImplId { domain, name, date },
    )(input)
}

fn compound_args(input: &[u8]) -> IResult<&[u8], CompoundArgs> {
    map(
        tuple((
            utf8str_cs::<{ u32::MAX }>,
            be_u32,
            variable_length_array::<_, _, _, { u32::MAX }>(nfs_argop),
        )),
        |(tag, minorversion, argarray)| CompoundArgs {
            tag,
            minorversion,
            argarray,
        },
    )(input)
}

fn exchange_id_args(input: &[u8]) -> IResult<&[u8], ExchangeIdArgs> {
    let (input, clientowner) = client_owner(input)?;
    let (input, flags) = be_u32(input)?;
    let (input, state_protect) = state_protect_args()(input)?;
    let (input, client_impl_id) = variable_length_array::<_, _, _, 1>(nfs_impl_id)(input)?;
    let client_impl_id = client_impl_id.into_iter().next();
    Ok((
        input,
        ExchangeIdArgs {
            clientowner,
            flags,
            state_protect,
            client_impl_id,
        },
    ))
}

fn nfs_argop(input: &[u8]) -> IResult<&[u8], NfsArgOp> {
    flat_map(nfs_opnum, |opnum| match opnum {
        NfsOpnum::OP_EXCHANGE_ID => {
            move |input| map(exchange_id_args, NfsArgOp::OP_EXCHANGE_ID)(input)
        }
        _ => todo!(),
    })(input)
}

fn state_protect_how(input: &[u8]) -> IResult<&[u8], StateProtectHow> {
    map_opt(be_u32, StateProtectHow::from_u32)(input)
}

fn state_protect_args<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], StateProtectArgs<'a>> {
    move |input: &'a [u8]| {
        let (input, state_protect_how) = state_protect_how(input)?;
        Ok(match state_protect_how {
            StateProtectHow::SP4_NONE => (input, StateProtectArgs::SP4_NONE),
            StateProtectHow::SP4_MACH_CRED => {
                let (input, mach_ops) = state_protect_ops(input)?;
                (input, StateProtectArgs::SP4_MACH_CRED(mach_ops))
            }
            StateProtectHow::SP4_SSV => {
                let (input, ssv_parms) = ssv_sp_parms(input)?;
                (input, StateProtectArgs::SP4_SSV(ssv_parms))
            }
        })
    }
}

pub fn rpc_msg(input: &[u8]) -> IResult<&[u8], RpcMessage> {
    map(tuple((be_u32, msg)), |(xid, message)| RpcMessage {
        xid,
        message,
    })(input)
}

fn msg(input: &[u8]) -> IResult<&[u8], Message> {
    flat_map(be_u32, |tag| match tag {
        0 => move |input| map(call, Message::Call)(input),
        1 => move |input| map(reply, Message::Reply)(input),
        _ => move |input| fail(input),
    })(input)
}

fn call(input: &[u8]) -> IResult<&[u8], Call> {
    let (input, _) = verify(be_u32, |&rpcvers| rpcvers == 2)(input)?;
    let (input, _) = verify(be_u32, |&prog| prog == 100003)(input)?;
    let (input, _) = verify(be_u32, |&vers| vers == 4)(input)?;
    let (input, proc) = be_u32(input)?;
    let (input, cred) = opaque_auth(input)?;
    let (input, verf) = opaque_auth(input)?;
    let (input, procedure) = procedure_call(proc)(input)?;
    Ok((
        input,
        Call {
            cred,
            verf,
            procedure,
        },
    ))
}

fn reply(input: &[u8]) -> IResult<&[u8], Reply> {
    flat_map(be_u32, |tag| match tag {
        0 => move |input| map(accepted_reply, Reply::Accepted)(input),
        1 => move |input| map(rejected_reply, Reply::Rejected)(input),
        _ => move |input| fail(input),
    })(input)
}

fn accepted_reply(input: &[u8]) -> IResult<&[u8], AcceptedReply> {
    map(tuple((opaque_auth, accepted_reply_body)), |(verf, body)| {
        AcceptedReply { verf, body }
    })(input)
}

fn accepted_reply_body(input: &[u8]) -> IResult<&[u8], AcceptedReplyBody> {
    flat_map(accept_status, |status| match status {
        AcceptStatus::Success => map(procedure_reply, AcceptedReplyBody::Success),
        _ => todo!(),
    })(input)
}

fn procedure_reply(input: &[u8]) -> IResult<&[u8], ProcedureReply> {
    length_value(
        be_u32,
        flat_map(be_u32, |proc| match proc {
            0 => move |input| Ok((input, ProcedureReply::Null)),
            1 => move |input| map(compound_result, ProcedureReply::Compound)(input),
            _ => move |input| fail(input),
        }),
    )(input)
}

fn compound_result(input: &[u8]) -> IResult<&[u8], CompoundResult> {
    todo!()
}

fn accept_status(input: &[u8]) -> IResult<&[u8], AcceptStatus> {
    map_opt(be_u32, AcceptStatus::from_u32)(input)
}

fn rejected_reply(input: &[u8]) -> IResult<&[u8], RejectedReply> {
    todo!()
}

fn opaque_auth(input: &[u8]) -> IResult<&[u8], OpaqueAuth> {
    map(
        tuple((auth_flavor, variable_length_opaque::<{ u32::MAX }>)),
        |(flavor, body)| OpaqueAuth { flavor, body },
    )(input)
}

fn auth_flavor(input: &[u8]) -> IResult<&[u8], AuthFlavor> {
    map_opt(be_u32, AuthFlavor::from_u32)(input)
}

fn procedure_call<'a>(proc: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ProcedureCall<'a>> {
    move |input: &'a [u8]| match proc {
        0 => Ok((input, ProcedureCall::Null)),
        1 => map(compound_args, ProcedureCall::Compound)(input),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_length_bitmap() {
        let input = &[0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03];
        let (input, result) = bitmap(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, Bitmap::from(vec![0x00010203]));
    }

    #[test]
    fn test_nfs_opnum() {
        let input = &[0x00, 0x00, 0x00, 0x03];
        let (input, result) = nfs_opnum(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, NfsOpnum::OP_ACCESS);
    }

    #[test]
    fn test_nfsstat() {
        let input = &[0x00, 0x00, 0x27, 0x39];
        let (input, result) = nfsstat(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, NfsStat::NFS4ERR_BADNAME);
    }
}
