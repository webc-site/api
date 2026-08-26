import { $ as $E } from "@1-/proto/E.js";
import auth$Empty from "./EmptyE.js";
import auth$InfoReq from "./InfoReqE.js";
import auth$UserReq from "./UserReqE.js";
import auth$mail$ChangeApplyReq from "./mail/ChangeApplyReqE.js";
import auth$mail$ChangeReq from "./mail/ChangeReqE.js";
import auth$mail$PasswordResetReq from "./mail/PasswordResetReqE.js";
import auth$mail$SigninReq from "./mail/SigninReqE.js";
import auth$mail$UserNewApplyReq from "./mail/UserNewApplyReqE.js";
import auth$mail$UserNewReq from "./mail/UserNewReqE.js";
import auth$token$DisableReq from "./token/DisableReqE.js";
import auth$token$EnableReq from "./token/EnableReqE.js";
import auth$token$LsReq from "./token/LsReqE.js";
import auth$token$NameSetReq from "./token/NameSetReqE.js";
import auth$token$NewReq from "./token/NewReqE.js";
import auth$token$RefreshReq from "./token/RefreshReqE.js";
import auth$token$RmReq from "./token/RmReqE.js";
import auth$user$BidRmReq from "./user/BidRmReqE.js";
import auth$user$ExitReq from "./user/ExitReqE.js";
import auth$user$NameSetReq from "./user/NameSetReqE.js";
import auth$user$PasswordSetReq from "./user/PasswordSetReqE.js";
import auth$user$TouchReq from "./user/TouchReqE.js";
export default $E([
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
