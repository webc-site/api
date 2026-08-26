import { $ as $D } from "@1-/proto/D.js";
import auth$Empty from "./EmptyD.js";
import auth$InfoReq from "./InfoReqD.js";
import auth$UserReq from "./UserReqD.js";
import auth$mail$ChangeApplyReq from "./mail/ChangeApplyReqD.js";
import auth$mail$ChangeReq from "./mail/ChangeReqD.js";
import auth$mail$PasswordResetReq from "./mail/PasswordResetReqD.js";
import auth$mail$SigninReq from "./mail/SigninReqD.js";
import auth$mail$UserNewApplyReq from "./mail/UserNewApplyReqD.js";
import auth$mail$UserNewReq from "./mail/UserNewReqD.js";
import auth$token$DisableReq from "./token/DisableReqD.js";
import auth$token$EnableReq from "./token/EnableReqD.js";
import auth$token$LsReq from "./token/LsReqD.js";
import auth$token$NameSetReq from "./token/NameSetReqD.js";
import auth$token$NewReq from "./token/NewReqD.js";
import auth$token$RefreshReq from "./token/RefreshReqD.js";
import auth$token$RmReq from "./token/RmReqD.js";
import auth$user$BidRmReq from "./user/BidRmReqD.js";
import auth$user$ExitReq from "./user/ExitReqD.js";
import auth$user$NameSetReq from "./user/NameSetReqD.js";
import auth$user$PasswordSetReq from "./user/PasswordSetReqD.js";
import auth$user$TouchReq from "./user/TouchReqD.js";
export default $D([
  /* 1 get */ auth$Empty,
  /* 2 info */ auth$InfoReq,
  /* 3 lang */ auth$Empty,
  /* 4 mail_change */ auth$mail$ChangeReq,
  /* 5 mail_change_apply */ auth$mail$ChangeApplyReq,
  /* 6 mail_password_reset */ auth$mail$PasswordResetReq,
  /* 7 mail_signin */ auth$mail$SigninReq,
  /* 8 mail_user_new */ auth$mail$UserNewReq,
  /* 9 mail_user_new_apply */ auth$mail$UserNewApplyReq,
  /* 10 token_disable */ auth$token$DisableReq,
  /* 11 token_enable */ auth$token$EnableReq,
  /* 12 token_ls */ auth$token$LsReq,
  /* 13 token_name_set */ auth$token$NameSetReq,
  /* 14 token_new */ auth$token$NewReq,
  /* 15 token_refresh */ auth$token$RefreshReq,
  /* 16 token_rm */ auth$token$RmReq,
  /* 17 user */ auth$UserReq,
  /* 18 user_bid_rm */ auth$user$BidRmReq,
  /* 19 user_exit */ auth$user$ExitReq,
  /* 20 user_name_set */ auth$user$NameSetReq,
  /* 21 user_password_set */ auth$user$PasswordSetReq,
  /* 22 user_touch */ auth$user$TouchReq
]);
