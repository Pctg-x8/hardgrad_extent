// Quaternion Library

vec4 qConjugate(vec4 iq) { return vec4(-iq.xyz, iq.w); }
vec4 qMult(vec4 q1, vec4 q2) { return vec4(cross(q1.xyz, q2.xyz) + q2.w * q1.xyz + q1.w * q2.xyz, q1.w * q2.w - dot(q1.xyz, q2.xyz)); }
vec4 qRot(vec3 in_vec, vec4 rq)
{
    vec4 q1 = rq;
    vec4 qp = vec4(in_vec, 0.0f);
    vec4 q2 = qConjugate(rq);
    vec4 qt = qMult(q1, qp);
    return qMult(qt, q2);
}
