use crate::*;

// Some Rust features are not supported, and are ignored by `minimize`.
// Those can be found by grepping "IGNORED".

pub fn translate_bb<'tcx>(bb: &rs::BasicBlockData<'tcx>, fcx: &mut FnCtxt<'tcx>) -> BasicBlock {
    let mut statements = List::new();
    for stmt in bb.statements.iter() {
        // unsupported statements will be IGNORED.
        if let Some(x) = translate_stmt(stmt, fcx) {
            statements.push(x);
        }
    }
    BasicBlock {
        statements,
        terminator: translate_terminator(bb.terminator(), fcx),
    }
}

fn translate_stmt<'tcx>(stmt: &rs::Statement<'tcx>, fcx: &mut FnCtxt<'tcx>) -> Option<Statement> {
    Some(match &stmt.kind {
        rs::StatementKind::Assign(box (place, rval)) => {
            Statement::Assign {
                destination: translate_place(place, fcx),
                source: translate_rvalue(rval, fcx)?, // assign of unsupported rvalues are IGNORED.
            }
        },
        rs::StatementKind::StorageLive(local) => {
            Statement::StorageLive(fcx.local_name_map[&local])
        },
        rs::StatementKind::StorageDead(local) => {
            Statement::StorageDead(fcx.local_name_map[&local])
        },
        rs::StatementKind::Deinit(..) | rs::StatementKind::Retag(..) => return None, // IGNORED for now.
        x => {
            dbg!(x);
            todo!()
        }
    })
}

fn translate_terminator<'tcx>(terminator: &rs::Terminator<'tcx>, fcx: &mut FnCtxt<'tcx>) -> Terminator {
    match &terminator.kind {
        rs::TerminatorKind::Return => Terminator::Return,
        rs::TerminatorKind::Goto { target } => Terminator::Goto(fcx.bb_name_map[&target]),
        rs::TerminatorKind::Call { func, target, destination, args, .. } => translate_call(fcx, func, args, destination, target),
        rs::TerminatorKind::Assert { target, .. } => { // Assert is IGNORED as of now.
            Terminator::Goto(fcx.bb_name_map[&target])
        }
        rs::TerminatorKind::SwitchInt { discr, targets, switch_ty: _ } => {
            assert!(discr.ty(&fcx.body, fcx.tcx).is_bool()); // for now we only support bool branching.

            let condition = translate_operand(discr, fcx);
            let then_block = targets.target_for_value(1);
            let then_block = fcx.bb_name_map[&then_block];

            let else_block = targets.target_for_value(0);
            let else_block = fcx.bb_name_map[&else_block];

            Terminator::If {
                condition,
                then_block,
                else_block,
            }
        }
        x => {
            dbg!(x);
            todo!()
        }
    }
}

fn translate_call<'tcx>(fcx: &mut FnCtxt<'tcx>, func: &rs::Operand<'tcx>, args: &[rs::Operand<'tcx>], destination: &rs::Place<'tcx>, target: &Option<rs::BasicBlock>) -> Terminator {
    let rs::Operand::Constant(box f) = func else { panic!() };
    let rs::ConstantKind::Val(_, f) = f.literal else { panic!() };
    let rs::TyKind::FnDef(f, substs_ref) = f.kind() else { panic!() };
    let key = (*f, *substs_ref);

    if fcx.tcx.crate_name(f.krate).as_str() == "intrinsics" {
        let intrinsic = match fcx.tcx.item_name(*f).as_str() {
            "print" => Intrinsic::PrintStdout,
            "eprint" => Intrinsic::PrintStderr,
            "exit" => Intrinsic::Exit,
            name => panic!("unsupported intrinsic `{}`", name),
        };
        Terminator::CallIntrinsic {
            intrinsic,
            arguments: args.iter().map(|x| translate_operand(x, fcx)).collect(),
            ret: None,
            next_block: target.as_ref().map(|t| fcx.bb_name_map[t]),
        }
    } else {
        if !fcx.fn_name_map.contains_key(&key) {
            let fn_name = fcx.fn_name_map.len();
            let fn_name = FnName(Name::new(fn_name as _));
            fcx.fn_name_map.insert(key, fn_name);
        }
        Terminator::Call {
            callee: fcx.fn_name_map[&key],
            arguments: args.iter().map(|x| (translate_operand(x, fcx), arg_abi())).collect(),
            ret: Some((translate_place(&destination, fcx), arg_abi())),
            next_block: target.as_ref().map(|t| fcx.bb_name_map[t]),
        }
    }
}
