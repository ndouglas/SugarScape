#include <algorithm>
#include <array>
#include <cassert>
#include <chrono>
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <random>
#include <string>
#include <vector>
using namespace std;
struct Agent { int age,type,born,pos; double death,tau; bool retired=false; vector<int> peers; };
bool imitates(int r,int n,double tau){return n>0 && double(r)/n>=tau;}
struct World {
 mt19937_64 rng; vector<Agent>a; map<int,vector<int>>cohorts; double rational; bool spread,all; int t=0,elig=65,mandatory=0; long empty=0,decisions=0,births=0,truncated=0,emptyBirth=0; array<long,102> events{},exposure{}; long imEvents=0;
 World(int seed,double r,bool sp,bool al,int man):rng(seed),a(8100),rational(r),spread(sp),all(al),mandatory(man){for(int i=0;i<8100;i++)make(i,20+i/100);for(int i=0;i<8100;i++)network(i,false);}
 int integer(int lo,int hi){return uniform_int_distribution<int>(lo,hi)(rng);} double u(){return generate_canonical<double,53>(rng);}
 void make(int id,int age){auto &x=a[id];x.age=age;x.born=t-age;x.death=60+40*u();double q=u();x.type=q<rational?0:(q<rational+.05?1:2);x.tau=spread?.5+.5*u():.5;x.retired=false;x.peers.clear();auto &c=cohorts[x.born];x.pos=c.size();c.push_back(id);}
 void network(int id,bool newborn){auto &x=a[id];int size=integer(10,25),extent=integer(0,5);vector<int>pool;for(auto it=cohorts.lower_bound(x.born-extent);it!=cohorts.end()&&it->first<=x.born+extent;++it)for(int j:it->second)if(j!=id)pool.push_back(j);int n=min(size,int(pool.size()));for(int k=0;k<n;k++){int j=integer(k,int(pool.size())-1);swap(pool[k],pool[j]);x.peers.push_back(pool[k]);}if(newborn){births++;truncated+=n<size;emptyBirth+=n==0;}}
 void replace(int id){auto &x=a[id];int born=x.born;auto &c=cohorts[born];int last=c.back();c[x.pos]=last;a[last].pos=x.pos;c.pop_back();if(c.empty())cohorts.erase(born);make(id,20);network(id,true);}
 bool act(int id){auto &x=a[id];++x.age;if(x.age>=x.death){replace(id);return false;}if(x.retired||x.age<elig)return false;exposure[x.age]++;bool retire=mandatory && x.age>=mandatory;if(!retire){if(x.type==0)retire=true;else if(x.type==1)retire=u()<.5;else{int den=0,num=0;for(int j:x.peers)if(all||a[j].age>=elig){den++;num+=a[j].retired;}decisions++;empty+=den==0;retire=imitates(num,den,x.tau);}}if(retire){x.retired=true;events[x.age]++;imEvents+=x.type==2;}return retire;}
 void step(){++t;events.fill(0);exposure.fill(0);imEvents=0;vector<int>order;for(auto const &kv:cohorts){auto c=kv.second;shuffle(c.begin(),c.end(),rng);order.insert(order.end(),c.begin(),c.end());}for(int id:order)act(id);}
 pair<int,int>counts(){int n=0,r=0;for(auto const &x:a)if(x.age>=elig){n++;r+=x.retired;}return {r,n};}
};
int mode(array<long,102>const& counts){int m=-1;long best=0;for(int age=0;age<102;age++)if(counts[age]>best){best=counts[age];m=age;}return m;}
void fixtures(){assert(!imitates(0,0,0));assert(imitates(1,2,.5));assert(!imitates(1,3,.5));assert(!imitates(1,2,.5001));World w(7,.1,false,false,0);int i=0,j=100;w.a[i].age=64;w.a[i].death=100;w.a[i].type=0;w.act(i);assert(w.a[i].age==65&&w.a[i].retired&&w.events[65]==1&&w.exposure[65]==1);w.a[j].age=64;w.a[j].death=65;w.act(j);assert(w.a[j].age==20&&!w.a[j].retired);w.a[i].age=65;w.a[i].retired=false;w.a[i].type=2;w.a[i].peers={j,200};w.a[j].age=64;w.a[j].retired=false;w.a[200].age=65;w.a[200].retired=true;w.a[i].tau=.75;w.act(i);assert(w.a[i].retired);w.a[i].retired=false;w.all=true;w.act(i);assert(!w.a[i].retired);assert(w.a[j].age==64);w.a[i].age=64;w.a[i].death=65.01;w.a[i].retired=false;w.a[i].type=0;w.act(i);assert(w.a[i].age==65&&w.a[i].retired);w.act(i);assert(w.a[i].age==20&&!w.a[i].retired);array<long,102>x{};x[65]=x[70]=4;assert(mode(x)==65);cerr<<"fixtures passed\n";}
int main(){fixtures();ofstream trace("trace.csv"),out("seeds.csv"),age("ages.csv");trace<<"case,seed,t,elig,retired,eligible,share,mode10,event_count,elig_age_events,imitator_events,elig_age_exposure\n";out<<"case,seed,rounds,first95,first99,first100,switch,mode_at_switch,post95,post99,post100,final_share,final_mode10,final10_elig_event_fraction,imitator_events,empty_decisions,decisions,births,truncated_birth_networks,empty_birth_networks\n";age<<"case,seed,t,age,exposure,events\n";
 struct Case{string name;double r;bool sp,all,policy,fixed;};vector<Case>cases={{"base",.1,false,false,false,false},{"slow",.05,false,false,false,false},{"all",.1,false,true,false,false},{"policy_original",.05,false,false,true,false},{"policy_revised",.05,true,false,true,false},{"policy_revised_fixed100",.05,true,false,true,true}};
 for(auto const &c:cases){auto start=chrono::steady_clock::now();for(int seed=2001;seed<=2050;seed++){World w(seed,c.r,c.sp,c.all,c.policy?70:0);int sw=-1,modeSwitch=-1;array<int,3>first={-1,-1,-1},post={-1,-1,-1};array<double,3>levels={.95,.99,1};array<array<long,102>,10>history{};array<long,102>rolling{};long imTotal=0;double share=0;int end=c.policy?1000:600;for(int t=1;t<=end;t++){w.step();auto [ret,n]=w.counts();share=n?double(ret)/n:0;auto &h=history[(t-1)%10];for(int k=0;k<102;k++){rolling[k]+=w.events[k]-h[k];h[k]=w.events[k];}int m=mode(rolling);long ev=accumulate(w.events.begin(),w.events.end(),0L);imTotal+=w.imEvents;for(int k=0;k<3;k++){if(sw<0&&first[k]<0&&share>=levels[k])first[k]=t;if(sw>=0&&post[k]<0&&share>=levels[k])post[k]=t-sw;}trace<<c.name<<','<<seed<<','<<t<<','<<w.elig<<','<<ret<<','<<n<<','<<share<<','<<m<<','<<ev<<','<<w.events[w.elig]<<','<<w.imEvents<<','<<w.exposure[w.elig]<<'\n';if(seed==2001)for(int k=60;k<=100;k++)if(w.exposure[k]||w.events[k])age<<c.name<<','<<seed<<','<<t<<','<<k<<','<<w.exposure[k]<<','<<w.events[k]<<'\n';if(c.policy&&sw<0&&((c.fixed&&t==100)||(!c.fixed&&share>=.95))){sw=t;modeSwitch=m;w.elig=62;end=t+100;}}
 long total10=accumulate(rolling.begin(),rolling.end(),0L);out<<c.name<<','<<seed<<','<<w.t;for(int v:first)out<<','<<v;out<<','<<sw<<','<<modeSwitch;for(int v:post)out<<','<<v;out<<','<<share<<','<<mode(rolling)<<','<<(total10?double(rolling[w.elig])/total10:0)<<','<<imTotal<<','<<w.empty<<','<<w.decisions<<','<<w.births<<','<<w.truncated<<','<<w.emptyBirth<<'\n';}cerr<<c.name<<" complete in "<<chrono::duration<double>(chrono::steady_clock::now()-start).count()<<"s\n";}
}
