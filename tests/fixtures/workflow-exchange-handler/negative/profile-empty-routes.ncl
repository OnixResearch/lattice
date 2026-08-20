let base = import "../profile-valid.ncl" in
let Contract = import "../../../../contracts/workflow-exchange-handler-profile.ncl" in
(base & { routes = [] }) | Contract.HandlerProfile
