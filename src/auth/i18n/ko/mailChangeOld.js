// GEN BY tran.js
export default (it) => '이메일 수정: ' + it.from + ' → ' + it.to + ' (기존 이메일 확인 코드: ' + it.code + ' )\n\n계정 이메일 주소 수정을 신청 중입니다: ' + it.from + ' → ' + it.to + '\n이메일 확인 코드는 다음과 같습니다.\n\n' + it.token_str + '\n인증 코드는 24시간 동안 유효합니다.\n\n이메일 주소 수정을 신청하지 않으셨다면 본 이메일을 무시하시고 계정 보안에 유의하시기 바랍니다.';
