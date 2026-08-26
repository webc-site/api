DEFINE SEQUENCE authType BATCH 1000 START 1;
DEFINE SEQUENCE gid BATCH 1000 START 1;
DEFINE SEQUENCE host BATCH 1000 START 1;
DEFINE SEQUENCE mailHost BATCH 1000 START 1;
DEFINE SEQUENCE org BATCH 1000 START 1;
DEFINE SEQUENCE userAgent BATCH 1000 START 1;

DEFINE TABLE authType SCHEMAFULL;
DEFINE FIELD host ON authType TYPE record<host>;
DEFINE FIELD id ON authType DEFAULT type::record('authType', `sequence`::nextval('authType'));
DEFINE FIELD type ON authType TYPE int;
DEFINE INDEX host_type ON authType FIELDS host, type UNIQUE;

DEFINE TABLE host SCHEMAFULL;
DEFINE FIELD host ON host TYPE string;
DEFINE FIELD id ON host DEFAULT type::record('host', `sequence`::nextval('host'));
DEFINE FIELD org ON host TYPE record<org>;
DEFINE INDEX host ON host FIELDS host UNIQUE;

DEFINE TABLE mail SCHEMAFULL;
DEFINE FIELD host ON mail TYPE record<mailHost>;
DEFINE FIELD id ON mail DEFAULT type::record('mail', `sequence`::nextval('gid'));
DEFINE FIELD prefix ON mail TYPE string;
DEFINE INDEX host_prefix ON mail FIELDS host, prefix UNIQUE;

DEFINE TABLE mailHost SCHEMAFULL;
DEFINE FIELD host ON mailHost TYPE string;
DEFINE FIELD id ON mailHost DEFAULT type::record('mailHost', `sequence`::nextval('mailHost'));
DEFINE INDEX host ON mailHost FIELDS host UNIQUE;

DEFINE TABLE org SCHEMAFULL;
DEFINE FIELD id ON org DEFAULT type::record('org', `sequence`::nextval('org'));
DEFINE FIELD name ON org TYPE string;

DEFINE TABLE phone SCHEMAFULL;
DEFINE FIELD area ON phone TYPE int;
DEFINE FIELD id ON phone DEFAULT type::record('phone', `sequence`::nextval('gid'));
DEFINE FIELD num ON phone TYPE int;
DEFINE INDEX area_num ON phone FIELDS area, num UNIQUE;

DEFINE TABLE userAgent SCHEMAFULL;
DEFINE FIELD browser ON userAgent TYPE string;
DEFINE FIELD browserVer ON userAgent TYPE string;
DEFINE FIELD id ON userAgent DEFAULT type::record('userAgent', `sequence`::nextval('userAgent'));
DEFINE FIELD os ON userAgent TYPE string;
DEFINE FIELD osVer ON userAgent TYPE string;
DEFINE INDEX browser_browserVer_os_osVer ON userAgent FIELDS browser, browserVer, os, osVer UNIQUE;
