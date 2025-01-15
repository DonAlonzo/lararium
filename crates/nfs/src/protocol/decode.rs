use super::*;
use nom::{
    bytes::complete::take,
    combinator::{fail, flat_map, map, map_opt, map_res, verify},
    error::ParseError,
    multi::count,
    number::complete::{be_i64, be_u32, be_u64},
    sequence::{pair, tuple},
    IResult, Parser,
};
use num_traits::FromPrimitive;

// RFC 1831

fn string<'a>(n: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a str> {
    map_res(take(n), std::str::from_utf8)
}

fn auth_sys_parms(input: &[u8]) -> IResult<&[u8], AuthSysParms> {
    map(
        tuple((
            be_u32,
            flat_map(be_u32, string),
            be_u32,
            be_u32,
            variable_length_array(16, be_u32),
        )),
        |(stamp, machine_name, uid, gid, gids)| AuthSysParms {
            stamp,
            machine_name: machine_name.into(),
            uid,
            gid,
            gids,
        },
    )(input)
}

//

fn aligned<'a>(length: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    map(
        pair(take(length as usize), take((4 - (length as usize % 4)) % 4)),
        |(data, _)| data,
    )
}

fn utf8str_cs(input: &[u8]) -> IResult<&[u8], Utf8StrCs> {
    flat_map(be_u32, |length| {
        map(
            map_res(aligned(length), std::str::from_utf8),
            Utf8StrCs::from,
        )
    })(input)
}

fn utf8str_cis(input: &[u8]) -> IResult<&[u8], Utf8StrCis> {
    flat_map(be_u32, |length| {
        map(
            map_res(aligned(length), std::str::from_utf8),
            Utf8StrCis::from,
        )
    })(input)
}

fn component(input: &[u8]) -> IResult<&[u8], Component> {
    map(utf8str_cs, Component)(input)
}

fn opaque<'a>(length: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Opaque<'a>> {
    map(aligned(length), Opaque::from)
}

fn variable_length_opaque<'a>(limit: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Opaque<'a>> {
    flat_map(verify(be_u32, move |&length| length <= limit), |length| {
        opaque(length as u32)
    })
}

fn variable_length_array<'a, O, E: ParseError<&'a [u8]>, F>(
    limit: u32,
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<O>, E>
where
    F: Parser<&'a [u8], O, E> + Clone,
{
    move |input: &'a [u8]| {
        let (input, length) = verify(be_u32, |&length| length <= limit)(input)?;
        count(parser.clone(), length as usize)(input)
    }
}

fn bitmap(input: &[u8]) -> IResult<&[u8], Bitmap> {
    map(variable_length_array(u32::MAX, be_u32), Bitmap::from)(input)
}

fn client_id(input: &[u8]) -> IResult<&[u8], ClientId> {
    map(be_u64, ClientId)(input)
}

fn sequence_id(input: &[u8]) -> IResult<&[u8], SequenceId> {
    map(be_u32, SequenceId)(input)
}

fn session_id(input: &[u8]) -> IResult<&[u8], SessionId> {
    map_res(take(16usize), |bytes: &[u8]| {
        bytes.try_into().map(SessionId)
    })(input)
}

fn file_handle(input: &[u8]) -> IResult<&[u8], FileHandle> {
    map_res(take(128usize), |bytes: &[u8]| {
        bytes.try_into().map(FileHandle)
    })(input)
}

fn slot_id(input: &[u8]) -> IResult<&[u8], SlotId> {
    map(be_u32, SlotId)(input)
}

fn qop(input: &[u8]) -> IResult<&[u8], Qop> {
    map(be_u32, Qop)(input)
}

fn nfs_opnum(input: &[u8]) -> IResult<&[u8], NfsOpnum> {
    map_opt(be_u32, NfsOpnum::from_u32)(input)
}

fn error(input: &[u8]) -> IResult<&[u8], Option<Error>> {
    map_opt(be_u32, |code| {
        if code == 0 {
            Some(None)
        } else {
            Error::from_u32(code).map(Some)
        }
    })(input)
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

fn server_owner(input: &[u8]) -> IResult<&[u8], ServerOwner> {
    map(
        tuple((be_u64, variable_length_opaque(NFS4_OPAQUE_LIMIT))),
        |(minor_id, major_id)| ServerOwner { minor_id, major_id },
    )(input)
}

fn client_owner(input: &[u8]) -> IResult<&[u8], ClientOwner> {
    map(
        tuple((verifier, variable_length_opaque(NFS4_OPAQUE_LIMIT))),
        |(verifier, owner_id)| ClientOwner { verifier, owner_id },
    )(input)
}

fn time(input: &[u8]) -> IResult<&[u8], Time> {
    map(tuple((be_i64, be_u32)), |(seconds, nanoseconds)| Time {
        seconds,
        nanoseconds,
    })(input)
}

fn ssv_sp_parms(input: &[u8]) -> IResult<&[u8], SsvSpParms> {
    map(
        tuple((
            state_protect_ops,
            variable_length_array(u32::MAX, sec_oid),
            variable_length_array(u32::MAX, sec_oid),
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

fn ssv_prot_info(input: &[u8]) -> IResult<&[u8], SsvProtInfo> {
    todo!()
}

fn sec_oid(input: &[u8]) -> IResult<&[u8], SecOid> {
    map(variable_length_opaque(u32::MAX), SecOid)(input)
}

fn nfs_impl_id(input: &[u8]) -> IResult<&[u8], NfsImplId> {
    map(
        tuple((utf8str_cis, utf8str_cs, time)),
        |(domain, name, date)| NfsImplId { domain, name, date },
    )(input)
}

fn compound_args(input: &[u8]) -> IResult<&[u8], CompoundArgs> {
    map(
        tuple((
            utf8str_cs,
            be_u32,
            variable_length_array(u32::MAX, nfs_argop),
        )),
        |(tag, minorversion, argarray)| CompoundArgs {
            tag,
            minorversion,
            argarray: argarray.into(),
        },
    )(input)
}

fn compound_result(input: &[u8]) -> IResult<&[u8], CompoundResult> {
    map(
        tuple((
            error,
            utf8str_cs,
            variable_length_array(u32::MAX, nfs_resop),
        )),
        |(error, tag, resarray)| CompoundResult {
            error,
            tag,
            resarray,
        },
    )(input)
}

fn nfs_resop(input: &[u8]) -> IResult<&[u8], NfsResOp> {
    flat_map(nfs_opnum, |opnum| match opnum {
        NfsOpnum::GetFileHandle => {
            move |input| map(get_file_handle_result, NfsResOp::GetFileHandle)(input)
        }
        NfsOpnum::PutRootFileHandle => {
            move |input| map(put_root_file_handle_result, NfsResOp::PutRootFileHandle)(input)
        }
        NfsOpnum::ExchangeId => move |input| map(exchange_id_result, NfsResOp::ExchangeId)(input),
        NfsOpnum::CreateSession => {
            move |input| map(create_session_result, NfsResOp::CreateSession)(input)
        }
        NfsOpnum::DestroySession => {
            move |input| map(destroy_session_result, NfsResOp::DestroySession)(input)
        }
        NfsOpnum::DestroyClientId => {
            move |input| map(destroy_client_id_result, NfsResOp::DestroyClientId)(input)
        }
        NfsOpnum::Sequence => move |input| map(sequence_result, NfsResOp::Sequence)(input),
        NfsOpnum::ReclaimComplete => {
            move |input| map(reclaim_complete_result, NfsResOp::ReclaimComplete)(input)
        }
        _ => todo!(),
    })(input)
}

// Operation 10: GETFH

fn get_file_handle_result(input: &[u8]) -> IResult<&[u8], GetFileHandleResult> {
    flat_map(error, |error| match error {
        None => move |input| map(get_file_handle_result_ok, GetFileHandleResult::Ok)(input),
        _ => todo!(),
    })(input)
}

fn get_file_handle_result_ok(input: &[u8]) -> IResult<&[u8], GetFileHandleResultOk> {
    map(file_handle, |object| GetFileHandleResultOk { object })(input)
}

// Operation 24: PUTROOTFS

fn put_root_file_handle_result(input: &[u8]) -> IResult<&[u8], PutRootFileHandleResult> {
    map(error, |error| PutRootFileHandleResult { error })(input)
}

// Operation 33: SECINFO

fn sec_info_args(input: &[u8]) -> IResult<&[u8], SecInfoArgs> {
    map(component, |name| SecInfoArgs { name })(input)
}

fn rpc_gss_svc(input: &[u8]) -> IResult<&[u8], RpcGssSvc> {
    map_opt(be_u32, RpcGssSvc::from_u32)(input)
}

fn rpc_sec_gss_info(input: &[u8]) -> IResult<&[u8], RpcSecGssInfo> {
    map(tuple((sec_oid, qop, rpc_gss_svc)), |(oid, qop, service)| {
        RpcSecGssInfo { oid, qop, service }
    })(input)
}

fn sec_info(input: &[u8]) -> IResult<&[u8], SecInfo> {
    todo!()
}

fn sec_info_result(input: &[u8]) -> IResult<&[u8], SecInfoResult> {
    todo!()
}

fn sec_info_result_ok(input: &[u8]) -> IResult<&[u8], SecInfoResultOk> {
    todo!()
}

// Operation 40

fn callback_sec_parms(input: &[u8]) -> IResult<&[u8], CallbackSecParms> {
    flat_map(auth_flavor, |auth_flavor| match auth_flavor {
        AuthFlavor::AuthNone => move |input| Ok((input, CallbackSecParms::AuthNone)),
        AuthFlavor::AuthSys => move |input| map(auth_sys_parms, CallbackSecParms::AuthSys)(input),
        _ => todo!("{auth_flavor:?} not implemented"),
    })(input)
}

// Operation 42: EXCHANGE_ID

fn exchange_id_flags(input: &[u8]) -> IResult<&[u8], ExchangeIdFlags> {
    map_opt(be_u32, ExchangeIdFlags::from_bits)(input)
}

fn exchange_id_args(input: &[u8]) -> IResult<&[u8], ExchangeIdArgs> {
    let (input, clientowner) = client_owner(input)?;
    let (input, flags) = exchange_id_flags(input)?;
    let (input, state_protect) = state_protect_args(input)?;
    let (input, client_impl_id) = variable_length_array(1, nfs_impl_id)(input)?;
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

fn exchange_id_result(input: &[u8]) -> IResult<&[u8], ExchangeIdResult> {
    flat_map(error, |error| match error {
        None => move |input| map(exchange_id_result_ok, ExchangeIdResult::Ok)(input),
        _ => fail,
    })(input)
}

fn exchange_id_result_ok(input: &[u8]) -> IResult<&[u8], ExchangeIdResultOk> {
    map(
        tuple((
            client_id,
            sequence_id,
            exchange_id_flags,
            state_protect_result,
            server_owner,
            variable_length_opaque(NFS4_OPAQUE_LIMIT),
            variable_length_array(1, nfs_impl_id),
        )),
        |(
            client_id,
            sequence_id,
            flags,
            state_protect,
            server_owner,
            server_scope,
            server_impl_id,
        )| ExchangeIdResultOk {
            client_id,
            sequence_id,
            flags,
            state_protect,
            server_owner,
            server_scope,
            server_impl_id: server_impl_id.into_iter().next(),
        },
    )(input)
}

// Operation 43: CREATE_SESSION

fn channel_attributes(input: &[u8]) -> IResult<&[u8], ChannelAttributes> {
    map(
        tuple((
            be_u32,
            be_u32,
            be_u32,
            be_u32,
            be_u32,
            be_u32,
            variable_length_array(1, be_u32),
        )),
        |(
            header_pad_size,
            max_request_size,
            max_response_size,
            max_response_size_cached,
            max_operations,
            max_requests,
            rdma_ird,
        )| ChannelAttributes {
            header_pad_size,
            max_request_size,
            max_response_size,
            max_response_size_cached,
            max_operations,
            max_requests,
            rdma_ird: rdma_ird.into_iter().next(),
        },
    )(input)
}

fn create_session_flags(input: &[u8]) -> IResult<&[u8], CreateSessionFlags> {
    map_opt(be_u32, CreateSessionFlags::from_bits)(input)
}

fn create_session_args(input: &[u8]) -> IResult<&[u8], CreateSessionArgs> {
    map(
        tuple((
            client_id,
            sequence_id,
            create_session_flags,
            channel_attributes,
            channel_attributes,
            be_u32,
            variable_length_array(u32::MAX, callback_sec_parms),
        )),
        |(
            client_id,
            sequence_id,
            flags,
            fore_channel_attributes,
            back_channel_attributes,
            cb_program,
            sec_parms,
        )| CreateSessionArgs {
            client_id,
            sequence_id,
            flags,
            fore_channel_attributes,
            back_channel_attributes,
            cb_program,
            sec_parms,
        },
    )(input)
}

fn create_session_result(input: &[u8]) -> IResult<&[u8], CreateSessionResult> {
    flat_map(error, |error| match error {
        None => move |input| map(create_session_result_ok, CreateSessionResult::Ok)(input),
        _ => fail,
    })(input)
}

fn create_session_result_ok(input: &[u8]) -> IResult<&[u8], CreateSessionResultOk> {
    map(
        tuple((
            session_id,
            sequence_id,
            create_session_flags,
            channel_attributes,
            channel_attributes,
        )),
        |(session_id, sequence_id, flags, fore_channel_attributes, back_channel_attributes)| {
            CreateSessionResultOk {
                session_id,
                sequence_id,
                flags,
                fore_channel_attributes,
                back_channel_attributes,
            }
        },
    )(input)
}

// Operation 44: DESTROY_SESSION

fn destroy_session_args(input: &[u8]) -> IResult<&[u8], DestroySessionArgs> {
    map(session_id, |session_id| DestroySessionArgs { session_id })(input)
}

fn destroy_session_result(input: &[u8]) -> IResult<&[u8], DestroySessionResult> {
    map(error, |error| DestroySessionResult { error })(input)
}

// Operation 52: SECINFO_NO_NAME

#[inline(always)]
fn sec_info_style(input: &[u8]) -> IResult<&[u8], SecInfoStyle> {
    map_opt(be_u32, SecInfoStyle::from_u32)(input)
}

#[inline(always)]
fn sec_info_no_name_args(input: &[u8]) -> IResult<&[u8], SecInfoNoNameArgs> {
    map(sec_info_style, SecInfoNoNameArgs)(input)
}

#[inline(always)]
fn sec_info_no_name_result(input: &[u8]) -> IResult<&[u8], SecInfoNoNameResult> {
    map(sec_info_result, SecInfoNoNameResult)(input)
}

// Operation 53: SEQUENCE

fn sequence_args(input: &[u8]) -> IResult<&[u8], SequenceArgs> {
    map(
        tuple((session_id, sequence_id, slot_id, slot_id, be_u32)),
        |(session_id, sequence_id, slot_id, highest_slot_id, cache_this)| SequenceArgs {
            session_id,
            sequence_id,
            slot_id,
            highest_slot_id,
            cache_this: cache_this != 0,
        },
    )(input)
}

fn sequence_status_flags(input: &[u8]) -> IResult<&[u8], SequenceStatusFlags> {
    map_opt(be_u32, SequenceStatusFlags::from_bits)(input)
}

fn sequence_result(input: &[u8]) -> IResult<&[u8], SequenceResult> {
    flat_map(error, |error| match error {
        None => move |input| map(sequence_result_ok, SequenceResult::Ok)(input),
        _ => fail,
    })(input)
}

fn sequence_result_ok(input: &[u8]) -> IResult<&[u8], SequenceResultOk> {
    map(
        tuple((
            session_id,
            sequence_id,
            slot_id,
            slot_id,
            slot_id,
            sequence_status_flags,
        )),
        |(
            session_id,
            sequence_id,
            slot_id,
            highest_slot_id,
            target_highest_slot_id,
            status_flags,
        )| SequenceResultOk {
            session_id,
            sequence_id,
            slot_id,
            highest_slot_id,
            target_highest_slot_id,
            status_flags,
        },
    )(input)
}

// Operation 57: DESTROY_CLIENT_ID

fn destroy_client_id_args(input: &[u8]) -> IResult<&[u8], DestroyClientIdArgs> {
    map(client_id, |client_id| DestroyClientIdArgs { client_id })(input)
}

fn destroy_client_id_result(input: &[u8]) -> IResult<&[u8], DestroyClientIdResult> {
    map(error, |error| DestroyClientIdResult { error })(input)
}

// Operation 58: RECLAIM_COMPLETE

fn reclaim_complete_args(input: &[u8]) -> IResult<&[u8], ReclaimCompleteArgs> {
    map(be_u32, |one_fs| ReclaimCompleteArgs {
        one_fs: one_fs != 0,
    })(input)
}

fn reclaim_complete_result(input: &[u8]) -> IResult<&[u8], ReclaimCompleteResult> {
    map(error, |error| ReclaimCompleteResult { error })(input)
}

//

fn nfs_argop(input: &[u8]) -> IResult<&[u8], NfsArgOp> {
    flat_map(nfs_opnum, |opnum| match opnum {
        NfsOpnum::GetFileHandle => move |input| Ok((input, NfsArgOp::GetFileHandle)),
        NfsOpnum::PutRootFileHandle => move |input| Ok((input, NfsArgOp::PutRootFileHandle)),
        NfsOpnum::ExchangeId => move |input| map(exchange_id_args, NfsArgOp::ExchangeId)(input),
        NfsOpnum::CreateSession => {
            move |input| map(create_session_args, NfsArgOp::CreateSession)(input)
        }
        NfsOpnum::DestroySession => {
            move |input| map(destroy_session_args, NfsArgOp::DestroySession)(input)
        }
        NfsOpnum::DestroyClientId => {
            move |input| map(destroy_client_id_args, NfsArgOp::DestroyClientId)(input)
        }
        NfsOpnum::SecInfoNoName => {
            move |input| map(sec_info_no_name_args, NfsArgOp::SecInfoNoName)(input)
        }
        NfsOpnum::Sequence => move |input| map(sequence_args, NfsArgOp::Sequence)(input),
        NfsOpnum::ReclaimComplete => {
            move |input| map(reclaim_complete_args, NfsArgOp::ReclaimComplete)(input)
        }
        _ => todo!("{opnum:?} not implemented"),
    })(input)
}

fn state_protect_how(input: &[u8]) -> IResult<&[u8], StateProtectHow> {
    map_opt(be_u32, StateProtectHow::from_u32)(input)
}

fn state_protect_args(input: &[u8]) -> IResult<&[u8], StateProtectArgs> {
    flat_map(state_protect_how, |how| match how {
        StateProtectHow::None => |input| Ok((input, StateProtectArgs::None)),
        StateProtectHow::MachineCredentials => {
            |input| map(state_protect_ops, StateProtectArgs::MachineCredentials)(input)
        }
        StateProtectHow::ServerSideValidation => {
            |input| map(ssv_sp_parms, StateProtectArgs::ServerSideValidation)(input)
        }
    })(input)
}

fn state_protect_result(input: &[u8]) -> IResult<&[u8], StateProtectResult> {
    flat_map(state_protect_how, |how| match how {
        StateProtectHow::None => |input| Ok((input, StateProtectResult::None)),
        StateProtectHow::MachineCredentials => {
            |input| map(state_protect_ops, StateProtectResult::MachineCredentials)(input)
        }
        StateProtectHow::ServerSideValidation => {
            |input| map(ssv_prot_info, StateProtectResult::ServerSideValidation)(input)
        }
    })(input)
}

pub fn message(input: &[u8]) -> IResult<&[u8], RpcMessage> {
    map(tuple((be_u32, message_type)), |(xid, message_type)| {
        RpcMessage { xid, message_type }
    })(input)
}

fn message_type(input: &[u8]) -> IResult<&[u8], MessageType> {
    map_opt(be_u32, MessageType::from_u32)(input)
}

pub fn call(input: &[u8]) -> IResult<&[u8], Call> {
    let (input, _) = verify(be_u32, |&rpcvers| rpcvers == 2)(input)?;
    let (input, _) = verify(be_u32, |&prog| prog == 100003)(input)?;
    let (input, _) = verify(be_u32, |&vers| vers == 4)(input)?;
    let (input, proc) = procedure_number(input)?;
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

pub fn reply(procedure_number: ProcedureNumber) -> impl FnMut(&[u8]) -> IResult<&[u8], Reply> {
    move |input| {
        let (input, tag) = be_u32(input)?;
        match tag {
            0 => map(accepted_reply(procedure_number), Reply::Accepted)(input),
            1 => map(rejected_reply(procedure_number), Reply::Rejected)(input),
            _ => fail(input),
        }
    }
}

fn accepted_reply(
    procedure_number: ProcedureNumber
) -> impl FnMut(&[u8]) -> IResult<&[u8], AcceptedReply> {
    move |input| {
        map(
            tuple((opaque_auth, accepted_reply_body(procedure_number))),
            |(verf, body)| AcceptedReply { verf, body },
        )(input)
    }
}

fn accepted_reply_body(
    procedure_number: ProcedureNumber
) -> impl FnMut(&[u8]) -> IResult<&[u8], AcceptedReplyBody> {
    move |input| {
        flat_map(accept_status, |status| match status {
            AcceptStatus::Success => map(
                procedure_reply(procedure_number),
                AcceptedReplyBody::Success,
            ),
            _ => todo!(),
        })(input)
    }
}

fn procedure_number(input: &[u8]) -> IResult<&[u8], ProcedureNumber> {
    map_opt(be_u32, ProcedureNumber::from_u32)(input)
}

fn procedure_reply(
    procedure_number: ProcedureNumber
) -> impl FnMut(&[u8]) -> IResult<&[u8], ProcedureReply> {
    move |input| match procedure_number {
        ProcedureNumber::Null => Ok((input, ProcedureReply::Null)),
        ProcedureNumber::Compound => map(compound_result, ProcedureReply::Compound)(input),
    }
}

fn accept_status(input: &[u8]) -> IResult<&[u8], AcceptStatus> {
    map_opt(be_u32, AcceptStatus::from_u32)(input)
}

fn rejected_reply(
    procedure_number: ProcedureNumber
) -> impl FnMut(&[u8]) -> IResult<&[u8], RejectedReply> {
    move |input| todo!()
}

fn opaque_auth(input: &[u8]) -> IResult<&[u8], OpaqueAuth> {
    map(
        tuple((auth_flavor, variable_length_opaque(u32::MAX))),
        |(flavor, body)| OpaqueAuth { flavor, body },
    )(input)
}

fn auth_flavor(input: &[u8]) -> IResult<&[u8], AuthFlavor> {
    map_opt(be_u32, AuthFlavor::from_u32)(input)
}

fn procedure_call<'a>(
    procedure_number: ProcedureNumber
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ProcedureCall<'a>> {
    move |input: &'a [u8]| match procedure_number {
        ProcedureNumber::Null => Ok((input, ProcedureCall::Null)),
        ProcedureNumber::Compound => map(compound_args, ProcedureCall::Compound)(input),
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
        assert_eq!(result, NfsOpnum::ACCESS);
    }

    #[test]
    fn test_error() {
        let input = &[0x00, 0x00, 0x27, 0x39];
        let (input, result) = error(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result, Some(Error::BADNAME));
    }

    #[test]
    fn test_utf8str_cs() {
        let input = &[
            0x00, 0x00, 0x00, 0x04, b'h', b'o', b'l', b'a', 0x00, 0x00, 0x00,
        ];
        let (input, result) = utf8str_cs(input).unwrap();
        assert_eq!(input, &[0x00, 0x00, 0x00]);
        assert_eq!(result.0, "hola");
    }

    #[test]
    fn test_utf8str_cs_alignment() {
        let input = &[
            0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00,
        ];
        let (input, result) = utf8str_cs(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result.0, "hello");
    }

    #[test]
    fn test_utf8str_cis() {
        let input = &[
            0x00, 0x00, 0x00, 0x04, b'h', b'o', b'l', b'a', 0x00, 0x00, 0x00,
        ];
        let (input, result) = utf8str_cis(input).unwrap();
        assert_eq!(input, &[0x00, 0x00, 0x00]);
        assert_eq!(result.0, "hola");
    }

    #[test]
    fn test_utf8str_cis_alignment() {
        let input = &[
            0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00,
        ];
        let (input, result) = utf8str_cis(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result.0, "hello");
    }

    #[test]
    fn test_opaque() {
        let input = &[0x01, 0x02, 0x03, 0x04];
        let (input, result) = opaque(4)(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result.0.into_owned(), &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_opaque_alignment() {
        let input = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x00, 0x00];
        let (input, result) = opaque(5)(input).unwrap();
        assert_eq!(input, &[]);
        assert_eq!(result.0.into_owned(), &[0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_exchange_id_result() {
        let input = &[
            0x0D, 0x99, 0xE9, 0xC6, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00,
            0x00, 0x00, 0xF0, 0xF2, 0x83, 0x67, 0x40, 0xD9, 0x35, 0x7A, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x66, 0x35, 0x30, 0x39, 0x30, 0x65, 0x65, 0x33,
            0x61, 0x34, 0x30, 0x35, 0x00, 0x00, 0x00, 0x0C, 0x66, 0x35, 0x30, 0x39, 0x30, 0x65,
            0x65, 0x33, 0x61, 0x34, 0x30, 0x35, 0x00, 0x00, 0x00, 0x00,
        ];
        let (input, message) = message(input).unwrap();
        let (input, reply) = reply(ProcedureNumber::Compound)(input).unwrap();
        assert_eq!(input, &[]);
    }

    #[test]
    fn test_secinfo_no_name_call() {
        let input = &[
            0x72, 0xf8, 0x13, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,
            0x86, 0xa3, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x64, 0x6f,
            0x6e, 0x61, 0x6c, 0x6f, 0x6e, 0x7a, 0x6f, 0x2d, 0x6c, 0x61, 0x70, 0x74, 0x6f, 0x70,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x35, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00,
        ];
        let (input, message) = message(input).unwrap();
        let (input, call) = call(input).unwrap();
        assert_eq!(input, &[]);
    }

    #[test]
    fn asd() {
        let input = &[
            0x72, 0xf8, 0x13, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x06,
            0x1A, 0xFA, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let (input, message) = message(input).unwrap();
        let (input, call) = call(input).unwrap();
        assert_eq!(input, &[]);
    }
}
